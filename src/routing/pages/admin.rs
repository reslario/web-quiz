use {
    serde::Serialize,
    serde_repr::Serialize_repr,
    rocket_contrib::{
        json::Json,
        templates::Template
    },
    rocket::{
        uri,
        get,
        put,
        post,
        delete,
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
            ops::QuestionId,
            models::{Category, Question, NewQuestion, NewCategory}
        },
    }
};

#[derive(Serialize)]
struct DisplayData {
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
pub fn admin(error: Option<RegisterError>, _guard: AdminGuard) -> Template {
    Template::render(
        if error.is_some() { "admin_error" } else { "admin" },
        DisplayData {
            error
        }
    )
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
pub struct FormQuestion {
    question: String,
    correct: String,
    incorrect1: String,
    incorrect2: String,
    incorrect3: String,
    category: i32,
}

#[post("/admin/add_question", data = "<form>")]
pub fn add_question(form: Form<FormQuestion>, _guard: AdminGuard, conn: DbConn) -> Result<Redirect, Status> {
    let FormQuestion {
        question, correct, incorrect1, incorrect2, incorrect3, category
    } = form.into_inner();

    let new = NewQuestion {
        category_id: category,
        string: &question,
        correct: &correct,
        incorrect: &[incorrect1, incorrect2, incorrect3]
    };

    Question::insert(&new, &conn)
        .map(|_| Redirect::to("/admin"))
        .or_500()
}

#[derive(FromForm)]
pub struct FormCategory {
    name: String
}

#[post("/admin/add_category", data = "<form>")]
pub fn add_category(form: Form<FormCategory>, _guard: AdminGuard, conn: DbConn) -> Result<Redirect, Status> {
    let new = NewCategory {
        name: &form.name
    };

    Category::insert(&new, &conn)
        .map(|_| Redirect::to("/admin"))
        .or_500()
}

pub mod api {
    use super::*;

    #[delete("/admin/delete_question/<id>")]
    pub fn delete_question(id: QuestionId, _guard: AdminGuard, conn: DbConn) -> Result<(), Status> {
        Question::delete(id, &conn)
            .or_500()
    }

    #[put("/admin/edit_question", data = "<question>")]
    pub fn edit_question(question: Json<Question>, _guard: AdminGuard, conn: DbConn) -> Result<(), Status> {
        let question = question.into_inner();
        let new = NewQuestion {
            category_id: *question.category_id(),
            string: &question.string,
            correct: &question.correct,
            incorrect: &question.incorrect
        };

        Question::update(question.id(), new, &conn)
            .map(drop)
            .or_500()
    }

    #[derive(Serialize)]
    pub struct JsonQuestions {
        questions: Vec<Question>
    }

    #[get("/admin/all_questions")]
    pub fn all_questions(_guard: AdminGuard, conn: DbConn) -> Result<Json<JsonQuestions>, Status> {
        Question::load_all(&conn)
            .map(|mut questions| {
                questions.sort_unstable_by_key(Question::id);
                questions
            })
            .map(|questions| JsonQuestions { questions })
            .map(Json)
            .or_500()
    }

    #[derive(Serialize)]
    pub struct JsonCategories {
        categories: Vec<Category>
    }

    #[get("/admin/all_categories")]
    pub fn all_categories(_guard: AdminGuard, conn: DbConn) -> Result<Json<JsonCategories>, Status> {
        Category::load_all(&conn)
            .map(|categories| JsonCategories { categories })
            .map(Json)
            .or_500()
    }
}