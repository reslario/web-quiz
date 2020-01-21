use std::path::{PathBuf, Path};
use rocket::{
    get,
    uri,
    response::NamedFile
};
use once_cell::sync::Lazy;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
static STATIC_CONTENT_PATH: Lazy<PathBuf> = Lazy::new(static_content_path);
static STATIC_PAGE_PATH: Lazy<PathBuf> = Lazy::new(static_page_path);

fn static_content_path() -> PathBuf {
    Path::new(ROOT)
        .join("content")
        .join("public")
        .join("static")
}

fn static_page_path() -> PathBuf {
    Path::new(ROOT)
        .join("content")
        .join("public")
        .join("html")
        .join("static")
}

#[get("/favicon.ico")]
pub fn favicon() -> std::io::Result<NamedFile> {
    static_content("favicon.ico".into())
}

#[get("/content/static/<path..>", rank = 5)]
pub fn static_content(path: PathBuf) -> std::io::Result<NamedFile> {
    NamedFile::open(
        STATIC_CONTENT_PATH.join(path)
    )
}

#[get("/<path..>")]
pub fn static_page(path: PathBuf) -> std::io::Result<NamedFile> {
    NamedFile::open(
        STATIC_PAGE_PATH.join(path)
    )
}

#[get("/")]
pub fn index() -> std::io::Result<NamedFile> {
    static_page("index.html".into())
}