use {
    owning_ref::MutexGuardRefMut,
    derive_more::{Deref, DerefMut},
    crate::models::{
        game::GameState,
        web::{Session, Admin, SESSION_COOKIE}
    },
    std::{
        sync::Mutex,
        collections::{HashMap, HashSet}
    },
    rocket::{
        State,
        Request,
        outcome::{Outcome, IntoOutcome},
        request::{self, FromRequest},
        http::{Status, Cookie}
    }
};

pub type GameStates = HashMap<Session, GameState>;
pub type SyncedGameStates = Mutex<GameStates>;

#[derive(Debug, Deref, DerefMut)]
pub struct SyncedGameState<'a>(MutexGuardRefMut<'a, GameStates, GameState>);

pub fn init_game_states() -> SyncedGameStates {
    Mutex::new(HashMap::new())
}

impl <'a, 'r> FromRequest<'a, 'r> for SyncedGameState<'a> {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
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
                    .and_then(|guard| MutexGuardRefMut::new(guard)
                        .try_map_mut(|states| states
                            .get_mut(&sess)
                            .ok_or(())
                        ).map(SyncedGameState)
                    ).into_outcome(Status::ServiceUnavailable)
                )
            )
    }
}

pub struct NewGameState<'a> {
    session: Session,
    game_states: &'a SyncedGameStates
}

impl <'a> NewGameState<'a> {
    pub fn set(self, state: GameState) {
        if let Ok(mut states) = self.game_states.lock() {
            states.insert(self.session, state);
        }
    }
}

impl <'a, 'r> FromRequest<'a, 'r> for NewGameState<'a> {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(NewGameState {
            session: Session::from_request(request)?,
            game_states: request
                .guard::<State<SyncedGameStates>>()
                .map_failure(|_| (Status::ServiceUnavailable, ()))
                .map(|state| state.inner())?
        })
    }
}

#[derive(Debug)]
pub struct EndGame {
    pub game_state: GameState
}

impl <'a, 'r> FromRequest<'a, 'r> for EndGame {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let end_game = request
            .guard::<Session>()
            .map(|sess| {
                request.guard::<State<SyncedGameStates>>()
                    .and_then(|states| states
                        .lock()
                        .map(|mut states| states
                            .remove(&sess)
                        ).map_err(drop)
                        .and_then(|state| state.ok_or(()))
                        .map(|game_state| EndGame { game_state })
                        .into_outcome(Status::InternalServerError)
                    )
            })?;

        request
            .cookies()
            .remove_private(Cookie::named(SESSION_COOKIE));

        end_game
    }
}

pub(super) type AdminSessions = HashSet<Admin>;
pub(super)  type SyncedAdminSessions = Mutex<AdminSessions>;

pub fn init_admin_sessions() -> SyncedAdminSessions {
    Mutex::new(HashSet::new())
}
