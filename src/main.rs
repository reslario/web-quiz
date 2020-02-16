#![feature(proc_macro_hygiene, decl_macro)]

// auto-generated schema.rs file won't compile without this
#[macro_use] extern crate diesel;

mod routing;
mod models;

use {
    rocket::{routes, catchers},
    rocket_contrib::templates::Template
};

fn main() {
    dotenv::dotenv().ok();

    rocket::ignite()
        .mount("/", routes![
            routing::index,
            routing::static_page,
            routing::static_content,
            routing::favicon,
            routing::pages::settings,
            routing::pages::new_game,
            routing::pages::answer,
            routing::pages::continue_game,
            routing::pages::end_game,
            routing::pages::use_joker,
            routing::pages::admin,
            routing::pages::verify,
            routing::pages::register,
            routing::pages::add_question
        ])
        .register(catchers![
            routing::catchers::unauthorized
        ])
        .attach(Template::fairing())
        .attach(models::db::DbConn::fairing())
        .manage(models::web::init_game_states())
        .manage(models::web::init_admin_sessions())
        .launch();
}
