use {
    rocket::FromForm,
    std::num::NonZeroU8,
    diesel::PgConnection,
    once_cell::sync::Lazy,
    rand::{thread_rng, Rng},
    argon2::{Config, Variant},
    crate::models::db::{
        AdminError,
        models::{Admin, NewAdmin}
    }
};

static CONFIG: Lazy<Config> = Lazy::new(config);

fn config() -> Config<'static> {
    Config {
        hash_length: 256,
        time_cost: 10,
        variant: Variant::Argon2id,
        ..Default::default()
    }
}

#[derive(Debug)]
pub enum Error {
    Hash(argon2::Error),
    Insert(AdminError)
}

#[derive(FromForm, Debug)]
pub struct Credentials {
    pub name: String,
    pub password: String
}

pub fn register(cred: &Credentials, conn: &PgConnection) -> Result<(), Error> {
    hash(&cred.password, &salt())
        .map_err(Error::Hash)
        .and_then(|hash| Admin::insert(
            &NewAdmin {
                name: &cred.name,
                password: &hash
            },
            conn
        ).map_err(Error::Insert)
        ).map(drop)
}

pub fn verify(cred: &Credentials, conn: &PgConnection) -> Result<bool, Error> {
    Admin::named(&cred.name, conn)
        .map_err(|e| Error::Insert(AdminError::Query(e)))
        .map(|admin| admin
            .map(|admin| verify_password(
                &cred.password,
                &admin.password,
            ))
            .unwrap_or(Ok(false))
            .map_err(Error::Hash)
        )?
}

fn verify_password(password: &str, hash: &str) -> argon2::Result<bool> {
    argon2::verify_encoded(
        hash,
        password.as_bytes(),
    )
}

fn hash(password: &str, salt: &str) -> argon2::Result<String> {
    argon2::hash_encoded(
        password.as_bytes(),
        salt.as_bytes(),
        &CONFIG
    )
}

fn salt() -> String {
    let mut rng = thread_rng();
    (0..128)
        .map(|_| rng.gen())
        .map(NonZeroU8::get)
        .map(char::from)
        .collect()
}