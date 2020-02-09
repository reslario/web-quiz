use rocket::{
    catch,
    response::Redirect
};

#[catch(401)]
pub fn unauthorized() -> Redirect {
    Redirect::to("/")
}