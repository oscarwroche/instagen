use axum::async_trait;
use std::{collections::HashMap, error::Error};

#[async_trait]
pub trait AuthService {
    async fn authenticate_user(
        &self,
        credentials: &AuthCredentials,
    ) -> Result<String, Box<dyn Error>>;
}

pub struct AuthCredentials {
    pub parameters: HashMap<String, String>,
}
