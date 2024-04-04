use crate::application::services::auth_service::AuthService;
use axum::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct AccessToken {
    access_token: String,
}

pub struct FacebookAuthService {
    app_id: String,
    client_secret: String,
}

impl FacebookAuthService {
    pub fn new(app_id: String, client_secret: String) -> Self {
        Self {
            app_id,
            client_secret,
        }
    }
}

pub const FB_OAUTH_REDIRECT_URL: &str = "https://127.0.0.1:8080";

#[async_trait]
impl AuthService for FacebookAuthService {
    async fn authenticate_user(
        &self,
        credentials: &crate::application::services::auth_service::AuthCredentials,
    ) -> Result<String, Box<dyn Error>> {
        let redirect_uri = format!("{}/", FB_OAUTH_REDIRECT_URL);
        // deserialize code from credentials
        let url = format!(
            "https://graph.facebook.com/v19.0/oauth/access_token?client_id={}&redirect_uri={}&client_secret={}&code={}",
            &self.app_id, redirect_uri, &self.client_secret, credentials.parameters.get("code").unwrap()
        );

        let response = Client::new().get(url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        let access_token = response.json::<AccessToken>().await?.access_token;
        println!("Access token: {}", access_token);

        Ok(access_token)
    }
}
