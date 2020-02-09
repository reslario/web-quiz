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
