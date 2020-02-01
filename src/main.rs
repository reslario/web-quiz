#![feature(proc_macro_hygiene, decl_macro)]

// auto-generated schema.rs file won't compile without this
#[macro_use] extern crate diesel;

use {
    rocket::routes,
    rocket_contrib::templates::Template
};
use diesel::Connection;

mod routing;
mod models;

fn main() {
    rocket::ignite()
        .mount("/", routes![
            routing::index,
            routing::static_page,
            routing::static_content,
            routing::favicon
        ])
        .attach(Template::fairing())
        .launch();
}
