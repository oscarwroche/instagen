use crate::{
    application::services::auth_service::{AuthCredentials, AuthService},
    domain::{entities::post::Post, repositories::post_repository::PostRepository},
};
use axum::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::{error::Error, sync::Arc};

pub const IG_POST_CAPTION: &str = "#berserk #darksouls #ai #guts #fantasy #darkfantasy #fantasyart #aiart #gutsberserk #lovecraft #undead #soulsborne";

#[derive(Deserialize)]
struct InstagramMediaContainer {
    id: String,
}

pub struct InstagramPostRepository {
    ig_user_id: String,
    auth_service: Arc<dyn AuthService + 'static + Sync + Send>,
    auth_token: Option<String>,
}

impl InstagramPostRepository {
    pub fn new(
        ig_user_id: String,
        auth_service: Arc<dyn AuthService + 'static + Sync + Send>,
    ) -> Self {
        Self {
            ig_user_id,
            auth_service,
            auth_token: None,
        }
    }

    pub async fn authenticate(
        &mut self,
        auth_credentials: AuthCredentials,
    ) -> Result<(), Box<dyn Error>> {
        let token = self
            .auth_service
            .authenticate_user(&auth_credentials)
            .await?;
        self.auth_token = Some(token);
        Ok(())
    }
}

#[async_trait]
impl PostRepository for InstagramPostRepository {
    async fn save(&self, post: &Post) -> Result<(), Box<dyn Error>> {
        let caption = IG_POST_CAPTION;

        let auth_token = self.auth_token.as_ref().unwrap();

        let media_request_url = format!(
            "https://graph.facebook.com/v19.0/{}/media?image_url={}&access_token={}&caption={}",
            self.ig_user_id, post.url, auth_token, caption
        );

        let container_id_response = Client::new().post(media_request_url).send().await?;

        // Ensure the request was successful (status code 2xx)
        if !container_id_response.status().is_success() {
            return Err(format!("Request failed: {}", container_id_response.text().await?).into());
        }

        let container_id = container_id_response
            .json::<InstagramMediaContainer>()
            .await?
            .id;

        let media_publish_request_url = format!(
            "https://graph.facebook.com/v19.0/{}/media_publish?creation_id={}&access_token={}",
            self.ig_user_id, container_id, auth_token,
        );

        Client::new().post(media_publish_request_url).send().await?;

        Ok(())
    }
}
