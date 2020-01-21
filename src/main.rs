#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{routes, get};
use rocket_contrib::templates::Template;

mod routing;

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
