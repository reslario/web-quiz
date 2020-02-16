use {
    uuid::Uuid,
    derive_more::Deref,
    owning_ref::MutexGuardRef,
    crate::models::web::{SyncedAdminSessions, Or500, AdminSessions},
    rocket::{
        State,
        Request,
        Outcome,
        Response,
        response::Responder,
        outcome::IntoOutcome,
        http::{Cookie, Status},
        request::{self, FromRequest}
    }
};

pub(super) const SESSION_COOKIE: &str = "session";

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

impl From<NewSession> for Session {
    fn from(new: NewSession) -> Self {
        new.0
    }
}

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

#[derive(Debug, Deref, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Admin(Session);

pub struct AdminGuard<'a>(MutexGuardRef<'a, AdminSessions, Admin>);

impl <'a, 'r> FromRequest<'a, 'r> for AdminGuard<'a> {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        request
            .guard::<State<SyncedAdminSessions>>()?
            .inner()
            .lock()
            .map_err(drop)
            .map(MutexGuardRef::new)
            .into_outcome(Status::InternalServerError)?
            .try_map(|admins| request
                .guard()
                .map(Admin)
                .success_or(())
                .and_then(|sess| admins
                    .get(&sess)
                    .ok_or(())
                )
            ).map(AdminGuard)
            .into_outcome(Status::Unauthorized)
    }
}

pub enum Login<R1, R2 = R1> {
    Success(R1),
    Failure(R2)
}

impl <'r, R1, R2> Responder<'r> for Login<R1, R2>
where
    R1: Responder<'r>,
    R2: Responder<'r>
{
    fn respond_to(self, request: &Request) -> Result<Response<'r>, Status> {
        match self {
            Login::Success(resp) => {
                add_admin_session(request)?;
                resp.respond_to(request)
            },
            Login::Failure(resp) => resp.respond_to(request)
        }
    }
}

fn add_admin_session(request: &Request) -> Result<(), Status> {
    request
        .guard::<State<SyncedAdminSessions>>()
        .success_or(Status::InternalServerError)?
        .lock()
        .or_500()?
        .insert(request
            .guard::<Session>()
            .or_500()
            .or_else(|_| request
                .guard::<NewSession>()
                .success_or(Status::InternalServerError)
                .map(Session::from)
            ).map(Admin)?
        );
    Ok(())
}
