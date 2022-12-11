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
        .expect("invalid timestamp")
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

#[derive(Debug)]
pub struct UnauthorizedError;

pub async fn get_auth_user(request: &Request<'_>) -> Result<User, UnauthorizedError> {
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
                    return Ok(user);
                }
                _ => {
                    return Err(UnauthorizedError);
                }
            }
        }
        _ => {
            return Err(UnauthorizedError);
        }
    }
}

pub struct AuthUser {
    pub user: User,
}

pub struct AuthAdminUser {
    pub user: User,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = UnauthorizedError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {

        let request_user = get_auth_user(request).await;

        match request_user {
            Ok(user) => {
                return Outcome::Success(AuthUser { user });
            }
            Err(err) => return Outcome::Failure((Status::Unauthorized, err))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthAdminUser {
    type Error = UnauthorizedError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {

        let request_user = get_auth_user(request).await;

        match request_user {
            Ok(user) => {
                if let Some(is_admin) = user.is_admin {
                    if is_admin == true{
                        return Outcome::Success(AuthAdminUser { user });
                    }
                }
                return Outcome::Failure((Status::Unauthorized, UnauthorizedError));
            }
            Err(err) => return Outcome::Failure((Status::Unauthorized, err))
        }
    }
}
