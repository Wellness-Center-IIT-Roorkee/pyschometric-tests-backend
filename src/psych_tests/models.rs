use std::error::Error;

use crate::diesel::prelude::*;
use rocket::fs::TempFile;
use rocket::serde::{Deserialize, Serialize};

use crate::schema::tests;
use crate::DBPool;

use super::utils::unsafe_save_file;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct PsychTest {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "tests"]
struct InsertablePsychTest {
    name: String,
    description: Option<String>,
    instructions: Option<String>,
    logo: Option<String>,
}

#[derive(FromForm)]
pub struct NewPsychTest<'r> {
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub logo: TempFile<'r>,
}

impl<'r> NewPsychTest<'r> {
    pub async fn save(self, db_pool: DBPool) -> Result<PsychTest, Box<dyn Error>> {
        let logo = unsafe_save_file(self.logo).await?;

        let insertable_psych_test = InsertablePsychTest {
            name: self.name,
            description: self.description,
            instructions: self.instructions,
            logo: Some(logo),
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
}

#[derive(Debug, Clone, AsChangeset)]
#[table_name = "tests"]
struct UpdatablePsychTest {
    name: Option<String>,
    description: Option<String>,
    instructions: Option<String>,
    logo: Option<String>,
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
