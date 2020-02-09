use {
    uuid::Uuid,
    crate::models::game::GameState,
    derive_more::{Deref, DerefMut},
    owning_ref::{MutexGuardRef, MutexGuardRefMut},
    std::{
        collections::HashMap,
        sync::{Arc, Mutex, MutexGuard}
    },
    rocket::{
        State,
        Request,
        outcome::{Outcome, IntoOutcome},
        request::{self, FromRequest},
        http::{Status, Cookie}
    }
};

const SESSION_COOKIE: &str = "session";

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Session {
    id: Uuid
}

impl Session {
    fn new() -> Session {
        Session {
            id: Uuid::new_v4()
        }
    }
}

impl <'a, 'r> FromRequest<'a, 'r> for Session {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        request
            .cookies()
            .get_private(SESSION_COOKIE)
            .as_ref()
            .map(Cookie::value)
            .ok_or(())
            .and_then(|val| Uuid::parse_str(val)
                .map(|id| Session { id })
                .map_err(drop)
            ).map(Outcome::Success)
            .unwrap_or(Outcome::Failure((Status::Unauthorized, ())))
    }
}

type GameStates = HashMap<Session, GameState>;
type SyncedGameStates = Arc<Mutex<GameStates>>;

#[derive(Debug, Deref, DerefMut)]
pub struct SyncedGameState<'a>(MutexGuardRef<'a, GameStates, GameState>);

#[derive(Debug, Deref, DerefMut)]
pub struct MutSyncedGameState<'a>(MutexGuardRefMut<'a, GameStates, GameState>);

pub fn init_game_states() -> SyncedGameStates {
    Arc::new(Mutex::new(HashMap::new()))
}

impl <'a, 'r> FromRequest<'a, 'r> for SyncedGameState<'a> {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        state_from_request(request, |states, sess| MutexGuardRef::new(states)
            .try_map(|states| states
                .get(&sess)
                .ok_or(())
            ).map(SyncedGameState)
        )
    }
}

impl <'a, 'r> FromRequest<'a, 'r> for MutSyncedGameState<'a> {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        state_from_request(request, |states, sess| MutexGuardRefMut::new(states)
            .try_map_mut(|states| states
                .get_mut(&sess)
                .ok_or(())
            ).map(MutSyncedGameState)
        )
    }
}

fn state_from_request<'a, 'r, T, F>(request: &'a Request<'r>, map: F) -> request::Outcome<T, ()>
where F: FnOnce(MutexGuard<'a, GameStates>, Session) -> Result<T, ()> {
    request
        .guard::<Session>()
        .map_failure(|_| (Status::Unauthorized, ()))
        .and_then(|sess| request
            .guard::<State<SyncedGameStates>>()
            .map_failure(|_| (Status::ServiceUnavailable, ()))
            .and_then(|states| states
                .inner()
                .lock()
                .map_err(drop)
                .and_then(|guard| map(guard, sess))
                .into_outcome(Status::ServiceUnavailable)
            )
        )
}

pub struct NewGameState {
    session: Session,
    game_states: Arc<Mutex<GameStates>>
}

impl NewGameState {
    pub fn set(self, state: GameState) {
        if let Ok(mut states) = self.game_states.lock() {
            states.insert(self.session, state);
        }
    }
}

impl <'a, 'r> FromRequest<'a, 'r> for NewGameState {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(NewGameState {
            session: Session::from_request(request)?,
            game_states: request
                .guard::<State<SyncedGameStates>>()
                .map_failure(|_| (Status::ServiceUnavailable, ()))
                .map(|states| Arc::clone(&*states))?
        })
    }
}

#[derive(Debug, Deref, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct NewSession(Session);

impl <'a, 'r> FromRequest<'a, 'r> for NewSession {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let sess = Session::new();
        request
            .cookies()
            .add_private(Cookie::new(SESSION_COOKIE, sess.id.to_string()));
        Outcome::Success(NewSession(sess))
    }
}

#[derive(Debug)]
pub struct EndSession;

impl <'a, 'r> FromRequest<'a, 'r> for EndSession {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        request
            .guard::<Session>()
            .map(|sess| {
                request.guard::<State<SyncedGameStates>>()
                    .and_then(|state| state
                        .lock()
                        .map(|mut state| state
                            .remove(&sess)
                            .map(drop)
                        ).map_err(drop)
                        .into_outcome(Status::InternalServerError)
                    )
            })?;

        request
            .cookies()
            .remove_private(Cookie::named(SESSION_COOKIE));
        Outcome::Success(EndSession)
    }
}

pub trait Or500<T> {
    fn or_500(self) -> Result<T, Status>;
}

impl <T, E> Or500<T> for Result<T, E> {
    fn or_500(self) -> Result<T, Status> {
        self.map_err(|_| Status::InternalServerError)
    }
}

impl <T> Or500<T> for Option<T> {
    fn or_500(self) -> Result<T, Status> {
        self.ok_or(Status::InternalServerError)
    }
}
