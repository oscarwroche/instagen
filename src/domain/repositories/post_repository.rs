use crate::domain::entities::post::Post;
use axum::async_trait;
use std::error::Error;

#[async_trait]
pub trait PostRepository {
    async fn save(&self, post: &Post) -> Result<(), Box<dyn Error>>; // Error handling simplified
}
