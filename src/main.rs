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
            routing::pages::start::settings,
            routing::pages::play::new_game,
            routing::pages::play::answer,
            routing::pages::play::continue_game,
            routing::pages::play::end_game,
            routing::pages::play::use_joker,
            routing::pages::play::resume,
            routing::pages::admin::admin,
            routing::pages::admin::verify,
            routing::pages::admin::register,
            routing::pages::admin::add_question
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
