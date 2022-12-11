use std::collections::HashMap;
use std::error::Error;

use crate::diesel::prelude::*;
use rocket::form::{self, Error as FormError};
use rocket::fs::TempFile;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::{serde_json, Json};
use rocket::serde::{Deserialize, Serialize};

use crate::schema::{questions, tests};
use crate::DBPool;

use super::utils::{unsafe_save_file, is_valid_range};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct PsychTest {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub logo: Option<String>,
    pub points_reference: serde_json::Value,
    pub points_interpretation: serde_json::Value,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "tests"]
struct InsertablePsychTest {
    name: String,
    description: Option<String>,
    instructions: Option<String>,
    logo: Option<String>,
    points_reference: serde_json::Value,
    points_interpretation: serde_json::Value,
}

#[derive(FromForm, Debug)]
pub struct NewPsychTest<'r> {
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub logo: TempFile<'r>,
    pub points_reference: Json<HashMap<i16, String>>,

    #[field(validate = validate_range_string())]
    pub points_interpretation: Json<HashMap<String, String>>,
}

fn validate_range_string<'v>(points_interpretation: &Json<HashMap<String, String>>) -> form::Result<'v, ()>{
    let interpretation_map = &points_interpretation.0;
    for (k, _) in interpretation_map {
        if is_valid_range(&k){
            continue;
        }
        Err(FormError::validation("invalid points interpretation format"))?;
    }
    Ok(())
}

impl<'r> NewPsychTest<'r> {
    pub async fn save(self, db_pool: DBPool) -> Result<PsychTest, Box<dyn Error>> {
        let logo = unsafe_save_file(self.logo).await?;

        let insertable_psych_test = InsertablePsychTest {
            name: self.name,
            description: self.description,
            instructions: self.instructions,
            logo: Some(logo),
            points_reference: json!(self.points_reference.into_inner()),
            points_interpretation: json!(self.points_interpretation.into_inner()),
        };
        let pysch_test: PsychTest = db_pool
            .run(move |conn| {
                diesel::insert_into(tests::table)
                    .values(&insertable_psych_test)
                    .get_result(conn)
            })
            .await?;
        Ok(pysch_test)
    }
}

#[derive(FromForm)]
pub struct UpdatePsychTest<'r> {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub logo: Option<TempFile<'r>>,
    pub points_reference: Option<Json<HashMap<i16, String>>>,

    #[field(validate = validate_optional_range_string())]
    pub points_interpretation: Option<Json<HashMap<String, String>>>,
}

fn validate_optional_range_string<'v>(points_interpretation: &Option<Json<HashMap<String, String>>>) -> form::Result<'v, ()>{
    match points_interpretation {
        Some(pi) => validate_range_string(pi),
        None => Ok(())
    }
}

#[derive(Debug, Clone, AsChangeset)]
#[table_name = "tests"]
struct UpdatablePsychTest {
    name: Option<String>,
    description: Option<String>,
    instructions: Option<String>,
    logo: Option<String>,
    points_reference: Option<serde_json::Value>,
    points_interpretation: Option<serde_json::Value>,
}

impl<'r> UpdatePsychTest<'r> {
    pub async fn save(self, id: i32, db_pool: DBPool) -> Result<PsychTest, Box<dyn Error>> {
        let mut logo: Option<String> = None;
        if let Some(file) = self.logo {
            logo = Some(unsafe_save_file(file).await?);
        }
        let updatable_psych_test = UpdatablePsychTest {
            name: self.name,
            description: self.description,
            instructions: self.instructions,
            logo,
            points_reference: match self.points_reference {
                Some(pr) => Some(json!(pr.into_inner())),
                None => None,
            },
            points_interpretation: match self.points_interpretation {
                Some(pi) => Some(json!(pi.into_inner())),
                None => None,
            },
        };
        let psych_test: PsychTest = db_pool
            .run(move |conn| {
                diesel::update(tests::table.find(id))
                    .set(&updatable_psych_test)
                    .get_result(conn)
            })
            .await?;

        // TODO: Delete previous logo if logo has been updated

        Ok(psych_test)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct Question {
    pub id: i32,
    pub test_id: i32,
    pub text: String,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "questions"]
pub struct NewQuestion {
    pub test_id: i32,
    pub text: String,
}

impl NewQuestion {
    pub async fn save(self, db_pool: &DBPool) -> Result<Question, Box<dyn Error>> {
        let question: Question = db_pool
            .run(move |conn| {
                diesel::insert_into(questions::table)
                    .values(&self)
                    .get_result(conn)
            })
            .await?;
        Ok(question)
    }
    pub async fn batch_save(
        questions: Vec<NewQuestion>,
        db_pool: &DBPool,
    ) -> Result<Vec<Question>, Box<dyn Error>> {
        let questions: Vec<Question> = db_pool
            .run(move |conn| {
                diesel::insert_into(questions::table)
                    .values(&questions)
                    .get_results(conn)
            })
            .await?;
        Ok(questions)
    }
}
