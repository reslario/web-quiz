#![feature(proc_macro_hygiene, decl_macro)]

// auto-generated schema.rs file won't compile without this
#[macro_use] extern crate diesel;

mod routing;
mod models;

use {
    rocket::routes,
    rocket_contrib::templates::Template
};

fn main() {
    dotenv::dotenv().ok();

    rocket::ignite()
        .mount("/", routes![
            routing::index,
            routing::static_page,
            routing::static_content,
            routing::favicon
        ])
        .attach(Template::fairing())
        .attach(models::db::DbConn::fairing())
        .manage(models::web::init_game_states())
        .launch();
}
