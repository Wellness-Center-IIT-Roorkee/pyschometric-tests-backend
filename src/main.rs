#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;
#[macro_use]
extern crate diesel;

use auth::routes as auth_routes;
use psych_tests::routes as test_routes;

mod auth;
mod psych_tests;
mod schema;
mod utils;

#[database("psychometric_tool")]
pub struct DBPool(diesel::PgConnection);

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    rocket::build()
        .mount("/", routes![auth_routes::login, auth_routes::whoami])
        .mount(
            "/tests",
            routes![
                test_routes::create_test,
                test_routes::update_test,
                test_routes::get_tests,
                test_routes::get_test
            ],
        )
        .attach(DBPool::fairing())
}
