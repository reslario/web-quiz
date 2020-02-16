use {
    serde::Serialize,
    diesel::QueryResult,
    rocket_contrib::templates::Template,
    rocket::{
        get,
        post,
        FromForm,
        http::Status,
        request::Form,
        response::Redirect
    },
    crate::models::{
        web::{AdminGuard, Login, Or500},
        account::{self, Credentials},
        db::{
            DbConn,
            models::{Category, Question, NewQuestion, NewCategory}
        },
    }
};

#[derive(Serialize)]
struct DisplayData<'a> {
    categories: &'a [Category]
}

#[get("/admin")]
pub fn admin(_guard: AdminGuard, conn: DbConn) -> Result<Template, Status> {
    Category::load_all(&conn)
        .as_ref()
        .map(|categories| DisplayData {
            categories
        })
        .map(|data| Template::render("admin", data))
        .or_500()
}

#[post("/login/verify", data = "<credentials>")]
pub fn verify(credentials: Form<Credentials>, conn: DbConn) -> Login<Redirect> {
    let success = account::verify(&credentials, &conn)
        .unwrap_or_default();

    if success {
        Login::Success(Redirect::to("/admin"))
    } else {
        Login::Failure(Redirect::to("/"))
    }
}

#[post("/admin/register", data = "<credentials>")]
pub fn register(credentials: Form<Credentials>, conn: DbConn) -> Result<Redirect, Status> {
    account::register(&credentials, &conn)
        .map(|_| Redirect::to("/admin"))
        .or_500()
}

