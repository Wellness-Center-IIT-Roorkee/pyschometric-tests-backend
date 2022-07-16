use crate::diesel::prelude::*;
use std::env;

use chrono::Utc;
use jsonwebtoken::{decode, encode, errors, DecodingKey, EncodingKey, Header, Validation};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    serde::{Deserialize, Serialize},
};

use crate::{schema::users, DBPool};

use super::models::User;

#[derive(Debug, Serialize, Deserialize)]

struct Claims {
    user: i32,
    exp: usize,
}
pub fn generate_jwt(user: &User) -> Result<String, errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(60))
        .expect("valid timestamp")
        .timestamp();
    let claims = Claims {
        user: user.id,
        exp: expiration as usize,
    };
    let secret: String = env::var("JWT_SECRET").unwrap();
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;
    Ok(token)
}

pub struct AuthUser {
    pub user: User,
}

#[derive(Debug)]
pub struct UnauthorizedError;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = UnauthorizedError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt = request.cookies().get_private("wellness_jwt");

        match jwt {
            Some(cookie) => {
                let jwt = cookie.value();
                let secret = env::var("JWT_SECRET").unwrap();
                let token = decode::<Claims>(
                    jwt,
                    &DecodingKey::from_secret(secret.as_ref()),
                    &Validation::default(),
                );
                match token {
                    Ok(token) => {
                        let pool = request.guard::<DBPool>().await.unwrap();
                        let user: User = pool
                            .run(move |conn| {
                                users::table
                                    .filter(users::id.eq(token.claims.user))
                                    .first(conn)
                            })
                            .await
                            .unwrap();
                        return Outcome::Success(AuthUser { user });
                    }
                    _ => {
                        return Outcome::Failure((Status::Unauthorized, UnauthorizedError));
                    }
                }
            }
            _ => {
                return Outcome::Failure((Status::Unauthorized, UnauthorizedError));
            }
        }
    }
}
