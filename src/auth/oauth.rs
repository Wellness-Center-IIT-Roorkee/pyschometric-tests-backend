use reqwest::{self, header};
use rocket::serde::{Deserialize, Serialize};
use std::{env, error::Error};

use crate::auth::constants::{OAUTH_ACCESS_TOKEN_ENDPOINT, OAUTH_USER_DATA_ENDPOINT};

#[derive(Serialize)]

struct TokenRequest<'a> {
    client_id: String,
    client_secret: String,
    grant_type: String,
    redirect_uri: String,
    code: &'a str,
}

impl<'a> TokenRequest<'a> {
    fn new(code: &str) -> TokenRequest {
        let client_id = env::var("OAUTH_CLIENT_ID").unwrap();
        let client_secret = env::var("OAUTH_CLIENT_SECRET").unwrap();
        let grant_type = env::var("OAUTH_GRANT_TYPE").unwrap();
        let redirect_uri = env::var("OAUTH_REDIRECT_URI").unwrap();
        TokenRequest {
            client_id,
            client_secret,
            grant_type,
            redirect_uri,
            code,
        }
    }
}

#[derive(Deserialize)]

struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthPerson {
    pub full_name: String,
    pub display_picture: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthContactInformation {
    pub institute_webmail_address: String,
    pub primary_phone_number: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthUserData {
    pub user_id: i64,
    pub person: OAuthPerson,
    pub contact_information: OAuthContactInformation,
}

pub async fn get_user_data(
    access_token: &str,
    client: &reqwest::Client,
) -> Result<OAuthUserData, Box<dyn Error>> {
    let mut auth_header = String::from("Bearer ");
    auth_header.push_str(access_token);
    let user_data = client
        .get(OAUTH_USER_DATA_ENDPOINT)
        .header(header::AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<OAuthUserData>()
        .await?;
    Ok(user_data)
}

pub async fn get_access_token(code: &str, client: &reqwest::Client) -> Option<String> {
    let request_params = TokenRequest::new(code);
    let response = client
        .post(OAUTH_ACCESS_TOKEN_ENDPOINT)
        .form(&request_params)
        .send()
        .await
        .unwrap();
    match response.error_for_status() {
        Ok(res) => {
            let token_response = res.json::<TokenResponse>().await.unwrap();
            Some(token_response.access_token)
        }
        Err(_) => {
            None
        }
    }
}
