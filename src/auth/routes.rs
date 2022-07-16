use reqwest;
use rocket::http::{Cookie, CookieJar};
use rocket::response::status::BadRequest;
use rocket::serde::{json::Json, Deserialize, Serialize};

use crate::auth::models;
use crate::auth::oauth;
use crate::auth::utils::AuthUser;
use crate::utils::ErrorJson;
use crate::DBPool;

use super::utils::generate_jwt;

#[derive(Deserialize, Serialize)]

pub struct LoginRequest {
    code: String,
}

#[post("/login", data = "<login_data>", format = "json")]
pub async fn login(
    pool: DBPool,
    login_data: Json<LoginRequest>,
    jar: &CookieJar<'_>,
) -> Result<Json<models::User>, BadRequest<Json<ErrorJson>>> {
    let client = reqwest::Client::new();
    let access_token = match oauth::get_access_token(&login_data.code, &client).await {
        Some(token) => token,
        None => {
            return Err(BadRequest(Some(Json(ErrorJson {
                error: "Access token authentication Failed".to_string(),
            }))));
        }
    };
    let user_data = oauth::get_user_data(&access_token, &client).await.unwrap();
    let user = models::NewUser::from_oauth_user_data(user_data)
        .save(pool)
        .await
        .unwrap();
    jar.add_private(Cookie::new("wellness_jwt", generate_jwt(&user).unwrap()));
    Ok(Json::<models::User>(user))
}

#[get("/whoami")]
pub fn whoami(authenticated_user: AuthUser) -> Json<models::User> {
    Json(authenticated_user.user)
}
