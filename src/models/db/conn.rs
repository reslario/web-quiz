#[rocket_contrib::database("db")]
pub struct DbConn(diesel::PgConnection);

pub type Connection = diesel::PgConnection;