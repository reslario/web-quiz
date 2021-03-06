use rocket::{
    Outcome,
    http::Status
};

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

impl <S, E, F> Or500<S> for Outcome<S, E, F> {
    fn or_500(self) -> Result<S, Status> {
        self.success_or(Status::InternalServerError)
    }
}