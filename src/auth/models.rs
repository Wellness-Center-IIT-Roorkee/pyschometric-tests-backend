use std::error::Error;

use crate::diesel::prelude::*;
use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};

use crate::auth::oauth;
use crate::schema::users::{self, channeli_id};
use crate::DBPool;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub channeli_id: i64,
    pub display_picture: Option<String>,

    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,

    #[serde(skip_deserializing)]
    pub is_admin: Option<bool>,
}

#[derive(AsChangeset, Debug, Clone, Deserialize, PartialEq, Serialize, Queryable, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub channeli_id: i64,
    pub display_picture: Option<String>,
}

impl NewUser {
    pub fn from_oauth_user_data(user_data: oauth::OAuthUserData) -> NewUser {
        NewUser {
            name: user_data.person.full_name,
            email: user_data.contact_information.institute_webmail_address,
            phone_number: if !user_data
                .contact_information
                .primary_phone_number
                .is_empty()
            {
                Some(user_data.contact_information.primary_phone_number)
            } else {
                None
            },
            channeli_id: user_data.user_id,
            display_picture: if !user_data.person.display_picture.is_empty() {
                Some(user_data.person.display_picture)
            } else {
                None
            },
        }
    }

    pub async fn save(self, db_pool: DBPool) -> Result<User, Box<dyn Error>> {
        let user: User = db_pool
            .run(move |conn| {
                diesel::insert_into(users::table)
                    .values(&self)
                    .on_conflict(channeli_id)
                    .do_update()
                    .set(&self)
                    .get_result(conn)
            })
            .await?;
        Ok(user)
    }
}
