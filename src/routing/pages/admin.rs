use {
    serde::Serialize,
    diesel::QueryResult,
    serde_repr::Serialize_repr,
    rocket_contrib::templates::Template,
    rocket::{
        uri,
        get,
        post,
        FromForm,
        response::Redirect,
        request::{Form, FromFormValue},
        http::{
            Status,
            RawStr,
            impl_from_uri_param_identity,
            uri::{Query, UriDisplay, Formatter}
        }
    },
    crate::models::{
        web::{AdminGuard, Login, Or500},
        account::{self, Credentials},
        db::{
            DbConn,
            AdminError,
            models::{Category, Question, NewQuestion, NewCategory}
        },
    }
};

#[derive(Serialize)]
struct DisplayData<'a> {
    categories: &'a [Category],
    error: Option<RegisterError>
}

#[derive(Serialize_repr, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RegisterError {
    None = 0,
    NameInUse = 1,
    Other = 2
}

impl RegisterError {
    pub fn id(self) -> u8 {
        self as u8
    }
}

impl <'v> FromFormValue<'v> for RegisterError {
    type Error = std::num::ParseIntError;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        form_value
            .parse::<u32>()
            .map(|int| match int {
                1 => RegisterError::NameInUse,
                2 => RegisterError::Other,
                _ => RegisterError::None
            })
    }
}

impl UriDisplay<Query> for RegisterError {
    fn fmt(&self, f: &mut Formatter<Query>) -> std::fmt::Result {
        f.write_value(self.id().to_string())
    }
}

impl_from_uri_param_identity!([Query] RegisterError);

#[get("/admin?<error>")]
pub fn admin(error: Option<RegisterError>, _guard: AdminGuard, conn: DbConn) -> Result<Template, Status> {
    Category::load_all(&conn)
        .as_ref()
        .map(|categories| Template::render(
            if error.is_some() { "admin_error" } else { "admin" },
            DisplayData {
                categories,
                error
            }
        ))
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
pub fn register(credentials: Form<Credentials>, conn: DbConn) -> Redirect {
    match account::register(&credentials, &conn) {
        Ok(()) => Redirect::to("/admin"),
        Err(e) => match e {
            account::Error::Hash(_) => Redirect::to(uri!(admin: RegisterError::Other)),
            account::Error::Insert(e) => match e {
                AdminError::Query(_) => Redirect::to(uri!(admin: RegisterError::Other)),
                AdminError::NameInUse => Redirect::to(uri!(admin: RegisterError::NameInUse))
            }
        }
    }
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