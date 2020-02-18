use {
    once_cell::sync::Lazy,
    std::path::{PathBuf, Path},
    rocket::{
        get,
        http::Status,
        response::NamedFile,
    }
};

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
pub fn favicon() -> Result<NamedFile, Status> {
    static_content("favicon.ico".into())
}

#[get("/content/static/<path..>", rank = 98)]
pub fn static_content(path: PathBuf) -> Result<NamedFile, Status> {
    NamedFile::open(STATIC_CONTENT_PATH.join(path))
        .map_err(|_| Status::NotFound)
}

#[get("/<path..>", rank = 99)]
pub fn static_page(path: PathBuf) -> Result<NamedFile, Status> {
    let mut path = STATIC_PAGE_PATH.join(path);
    path.set_extension("html");
    NamedFile::open(path)
        .map_err(|_| Status::NotFound)
}

#[get("/")]
pub fn index() -> Result<NamedFile, Status> {
    static_page("index.html".into())
}