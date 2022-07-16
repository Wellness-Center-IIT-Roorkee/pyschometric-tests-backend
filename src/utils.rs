use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]

pub struct ErrorJson {
    pub error: String,
}

// pub fn get_db_connection() -> PgConnection {
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
// }
