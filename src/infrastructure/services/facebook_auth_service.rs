use crate::application::services::auth_service::AuthService;
use axum::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::{env, error::Error};

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

#[async_trait]
impl AuthService for FacebookAuthService {
    async fn authenticate_user(
        &self,
        credentials: &crate::application::services::auth_service::AuthCredentials,
    ) -> Result<String, Box<dyn Error>> {
        let tcp_listener_address = env::var("TCP_LISTENER_ADDRESS").unwrap();
        let redirect_uri = format!("https://{}/api/perform_post_action", tcp_listener_address);
        let url = format!(
            "https://graph.facebook.com/v19.0/oauth/access_token?client_id={}&redirect_uri={}&client_secret={}&code={}",
            &self.app_id, redirect_uri, &self.client_secret, credentials.parameters.get("code").unwrap()
        );

        let response = Client::new().get(url).send().await?;

        if !response.status().is_success() {
            println!("Request failed: {}", response.text().await.unwrap());
            return Err(format!("FB auth failed").into());
        }

        let access_token = response.json::<AccessToken>().await?.access_token;

        Ok(access_token)
    }
}
