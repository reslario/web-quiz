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

#[derive(FromForm, Debug)]
pub struct AddQuestion {
    question: String,
    correct: String,
    incorrect1: String,
    incorrect2: String,
    incorrect3: String,
    category: Option<i32>,
    new_category: Option<String>
}

#[post("/admin/add_question", data = "<add_q>")]
pub fn add_question(add_q: Form<AddQuestion>, conn: DbConn) -> Result<Redirect, Status> {
    let (string, correct, incorrect) = (
        &add_q.question,
        &add_q.correct,
        &[
            add_q.incorrect1.clone(),
            add_q.incorrect2.clone(),
            add_q.incorrect3.clone()
        ]
    );

    add_q.category
        .map(|category_id| Ok(NewQuestion {
            category_id,
            string,
            correct,
            incorrect
        }))
        .or_else(|| add_q.new_category
            .as_ref()
            .map(|cat| new_question_and_category(cat, string, correct, incorrect, &conn))
        ).map(|res| res
            .and_then(|new_q| Question::insert(&new_q, &conn))
        ).or_500()?
        .or_500()
        .map(|_| Redirect::to("/admin"))
}

fn new_question_and_category<'a>(
    new_cat: &str,
    string: &'a str,
    correct: &'a str,
    incorrect: &'a [String; 3],
    conn: &DbConn
) -> QueryResult<NewQuestion<'a>> {
        Category::insert(
            &NewCategory {
                name: new_cat
            },
            &conn
        ).map(|cat| NewQuestion::with_category(
            &cat,
            string,
            correct,
            incorrect
        ))
}