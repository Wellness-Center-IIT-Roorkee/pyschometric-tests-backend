use diesel::RunQueryDsl;
use rocket::form::Form;
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};

use crate::diesel::prelude::*;
use crate::diesel::result::Error as DieselError;
use crate::schema::tests;
use crate::utils::ErrorJson;
use crate::DBPool;

use super::models;

#[derive(Deserialize, Serialize)]
pub struct PsychTests {
    items: Vec<models::PsychTest>,
}

#[get("/")]
pub async fn get_tests(pool: DBPool) -> Json<PsychTests> {
    let psych_tests = pool
        .run(move |conn| tests::table.load::<models::PsychTest>(conn))
        .await
        .unwrap();

    Json(PsychTests { items: psych_tests })
}

#[get("/<id>")]
pub async fn get_test(
    id: i32,
    pool: DBPool,
) -> Result<Json<models::PsychTest>, NotFound<Json<ErrorJson>>> {
    let psych_test = pool
        .run(move |conn| tests::table.filter(tests::id.eq(id)).first(conn))
        .await;

    match psych_test {
        Ok(test) => Ok(Json(test)),
        Err(err) => match err {
            DieselError::NotFound => Err(NotFound(Json(ErrorJson {
                error: "Requested Test not found.".to_string(),
            }))),
            other_error => panic!("{:#?}", other_error),
        },
    }
}

#[post("/", data = "<test_form>")]
pub async fn create_test(
    pool: DBPool,
    test_form: Form<models::NewPsychTest<'_>>,
) -> Json<models::PsychTest> {
    let psych_test = test_form.into_inner().save(pool);
    Json(psych_test.await.unwrap())
}

#[patch("/<id>", data = "<test_form>")]
pub async fn update_test(
    id: i32,
    pool: DBPool,
    test_form: Form<models::UpdatePsychTest<'_>>,
) -> Result<Json<models::PsychTest>, NotFound<Json<ErrorJson>>> {
    let psych_test = test_form.into_inner().save(id, pool).await;
    match psych_test {
        Ok(test) => Ok(Json(test)),
        Err(err) => {
            if let Some(e) = err.downcast_ref::<DieselError>() {
                match e {
                    DieselError::NotFound => Err(NotFound(Json(ErrorJson {
                        error: "Requested Test not found.".to_string(),
                    }))),
                    other_error => panic!("{:#?}", other_error),
                }
            } else {
                panic!("{:#?}", err)
            }
        }
    }
}
