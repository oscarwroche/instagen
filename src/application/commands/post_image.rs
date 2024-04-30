use crate::domain::{entities::post::Post, repositories::post_repository::PostRepository};
pub async fn post_image(post_repository: &(dyn PostRepository + Sync), image_url: String) {
    let post = Post { url: image_url };
    let _ = post_repository.save(&post).await;
}
