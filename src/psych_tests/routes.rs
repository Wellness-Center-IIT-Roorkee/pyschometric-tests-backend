use diesel::RunQueryDsl;
use rocket::form::Form;
use rocket::response::status::{Accepted, NotFound};
use rocket::serde::json::serde_json::json;
use rocket::serde::json::{serde_json, Json};
use rocket::serde::{Deserialize, Serialize};

use crate::auth::utils::{AuthAdminUser, AuthUser};
use crate::diesel::prelude::*;
use crate::diesel::result::Error as DieselError;
use crate::schema::{questions, tests};
use crate::utils::ErrorJson;
use crate::DBPool;

use super::models::{self, PsychTest};
use super::utils::parse_score_range;

#[derive(Deserialize, Serialize)]
pub struct PsychTests {
    items: Vec<models::PsychTest>,
}

#[derive(Deserialize, Serialize)]
pub struct TestQuestion {
    text: String,
}

#[derive(Deserialize, Serialize)]
pub struct TestQuestions {
    pub questions: Vec<TestQuestion>,
}

#[derive(FromForm, Deserialize)]
pub struct TestScore {
    score: i16,
}

#[get("/")]
pub async fn get_tests(_user: AuthUser, pool: DBPool) -> Json<PsychTests> {
    let psych_tests = pool
        .run(move |conn| tests::table.load::<models::PsychTest>(conn))
        .await
        .unwrap();

    Json(PsychTests { items: psych_tests })
}

#[get("/<id>")]
pub async fn get_test(
    id: i32,
    _user: AuthUser,
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
    _user: AuthAdminUser,
    pool: DBPool,
    test_form: Form<models::NewPsychTest<'_>>,
) -> Json<models::PsychTest> {
    let psych_test = test_form.into_inner().save(pool);
    Json(psych_test.await.unwrap())
}

#[patch("/<id>", data = "<test_form>")]
pub async fn update_test(
    id: i32,
    _user: AuthAdminUser,
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

#[get("/<test_id>/questions")]
pub async fn get_questions(
    _user: AuthUser,
    test_id: i32,
    pool: DBPool,
) -> Result<Json<Vec<models::Question>>, NotFound<Json<ErrorJson>>> {
    let test_questions = pool
        .run(move |conn| {
            questions::table
                .filter(questions::test_id.eq(test_id))
                .load::<models::Question>(conn)
        })
        .await;
    match test_questions {
        Ok(questions) => Ok(Json(questions)),
        Err(err) => match err {
            DieselError::NotFound => Err(NotFound(Json(ErrorJson {
                error: "Requested Test not found.".to_string(),
            }))),
            other_error => panic!("{:#?}", other_error),
        },
    }
}

#[post("/<test_id>/questions", data = "<test_questions>")]
pub async fn create_questions(
    _user: AuthAdminUser,
    test_id: i32,
    test_questions: Json<TestQuestions>,
    pool: DBPool,
) -> Json<Vec<models::Question>> {
    let test_questions = test_questions
        .into_inner()
        .questions
        .into_iter()
        .map(|question| models::NewQuestion {
            text: question.text,
            test_id,
        })
        .collect::<Vec<models::NewQuestion>>();
    let questions = models::NewQuestion::batch_save(test_questions, &pool)
        .await
        .unwrap();
    Json(questions)
}

#[delete("/<_test_id>/questions/<question_id>")]
pub async fn delete_question(
    _user: AuthAdminUser,
    _test_id: i32,
    question_id: i32,
    pool: DBPool,
) -> Result<Accepted<String>, NotFound<Json<ErrorJson>>> {
    let del = pool
        .run(move |conn| {
            diesel::delete(questions::table.filter(questions::id.eq(question_id))).execute(conn)
        })
        .await;

    match del {
        Ok(_) => return Ok(Accepted(Some("Question deleted successfully".to_string()))),
        Err(err) => match err {
            DieselError::NotFound => Err(NotFound(Json(ErrorJson {
                error: "Requested question not found.".to_string(),
            }))),
            other_error => panic!("{:#?}", other_error),
        },
    }
}

#[post("/<test_id>/evaluate", data = "<test_score>")]
pub async fn get_test_score(
    test_id: i32,
    _user: AuthAdminUser,
    test_score: Json<TestScore>,
    pool: DBPool,
) -> Result<serde_json::Value, NotFound<Json<ErrorJson>>> {
    let psych_test: Result<PsychTest, DieselError> = pool
        .run(move |conn| tests::table.filter(tests::id.eq(test_id)).first(conn))
        .await;

    let score = test_score.score as u32;

    match psych_test {
        Ok(test) => {
            let points_interpretation = test.points_interpretation.as_object();
            match points_interpretation {
                Some(pi) => {
                    for (k, v) in pi {
                        let point_range = parse_score_range(k);
                        match point_range {
                            Some(pr) => {
                                if score >= pr.0 && score <= pr.1 {
                                    return Ok(json!({ "result": v }));
                                }
                            }
                            None => panic!("Invalid test interpretation for the given test"),
                        }
                    }
                }
                None => panic!("Invalid test interpretation for the given test"),
            }
            // TODO: Send a 400 response instead of 200
            Ok(json!({"result": "Invalid test score"}))
        }
        Err(err) => match err {
            DieselError::NotFound => Err(NotFound(Json(ErrorJson {
                error: "Requested Test not found.".to_string(),
            }))),
            other_error => panic!("{:#?}", other_error),
        },
    }
}
