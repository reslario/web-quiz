#![feature(proc_macro_hygiene, decl_macro)]

// auto-generated schema.rs file won't compile without this
#[macro_use] extern crate diesel;

mod routing;
mod models;

use {
    rocket::{
        routes,
        catchers,
        fairing::Fairing
    },
    rocket_contrib::templates::Template
};

fn main() {
    dotenv::dotenv().ok();
    rocket().launch();
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![
            routing::index,
            routing::static_page,
            routing::static_content,
            routing::favicon,
            routing::pages::start::settings,
            routing::pages::play::new_game,
            routing::pages::play::answer,
            routing::pages::play::continue_game,
            routing::pages::play::end_game,
            routing::pages::play::use_joker,
            routing::pages::play::resume,
            routing::pages::play::failed,
            routing::pages::admin::admin,
            routing::pages::admin::verify,
            routing::pages::admin::register,
            routing::pages::admin::add_question,
            routing::pages::admin::add_category,
            routing::pages::admin::api::delete_question,
            routing::pages::admin::api::edit_question,
            routing::pages::admin::api::all_questions,
            routing::pages::admin::api::all_categories
        ])
        .register(catchers![
            routing::catchers::unauthorized
        ])
        .attach(templates())
        .attach(models::db::DbConn::fairing())
        .manage(models::web::init_game_states())
        .manage(models::web::init_admin_sessions())
}

fn templates() -> impl Fairing {
    Template::custom(|engines| {
        models::tera::configure(&mut engines.tera)
    })
}

#[cfg(test)]
mod test {
    use {
        std::sync::Mutex,
        once_cell::sync::Lazy,
        diesel::{PgConnection, Connection}
    };

    pub static CONN: Lazy<Mutex<PgConnection>> = Lazy::new(|| {
        dotenv::dotenv().ok();
        Mutex::new(
            PgConnection::establish(&std::env::var("DATABASE_URL").unwrap()).unwrap()
        )
    });

    // re-export the rocket initialiser so tests (and only tests) can use it
    pub fn rocket() -> rocket::Rocket {
        super::rocket()
    }
}
