#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;
#[macro_use]
extern crate diesel;

use auth::routes as auth_routes;

mod auth;
mod schema;
mod utils;

#[database("psychometric_tool")]
pub struct DBPool(diesel::PgConnection);

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    rocket::build()
        .mount("/", routes![auth_routes::login, auth_routes::whoami])
        .attach(DBPool::fairing())
}
