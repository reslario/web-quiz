use {
    serde::Serialize,
    rocket_contrib::templates::Template,
    crate::models::db,
    rocket::{
        post,
        FromForm,
        http::{Status, RawStr},
        request::{Form, FormItems, FromFormValue}
    }
};

#[derive(FromForm, Serialize)]
pub struct User {
    name: String
}

#[derive(Serialize)]
struct AvailableSettings {
    user: String,
    categories: Vec<db::models::Category>
}

#[post("/settings", data="<user>")]
pub fn settings(user: Form<User>, conn: db::DbConn) -> Result<Template, Status> {
    db::models::Category::load_all(&conn)
        .map_err(|_| Status::InternalServerError)
        .map(|categories| Template::render(
            "settings",
            &AvailableSettings {
                user: user.0.name,
                categories
            }
        ))
}


#[derive(Debug)]
pub struct Settings {
    pub user: String,
    pub categories: Vec<i32>
}

// rocket 4.x doesn't have support for multi-select forms yet,
// so this will have to do for now
impl <'f> rocket::request::FromForm<'f> for Settings {
    type Error = &'f RawStr;

    fn from_form(it: &mut FormItems<'f>, strict: bool) -> Result<Self, Self::Error> {
        let mut user = None;
        let mut categories = Vec::new();

        for (key, val) in it.map(|fi| fi.key_value()) {
            match key.url_decode_lossy().as_str() {
                "user" => user = Some(String::from_form_value(val)?),
                "categories" => categories.push(
                    i32::from_form_value(val)?
                ),
                _ if strict => return Err(val),
                _ => {}
            }
        }

        let user = user.ok_or(RawStr::from_str("user not specified"))?;

        Ok(Settings {
            user,
            categories
        })
    }
}