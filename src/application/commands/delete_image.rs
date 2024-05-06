use std::{error::Error, sync::Arc};

use crate::domain::repositories::image_repository::ImageRepository;
pub async fn delete_image(
    image_repository: Arc<dyn ImageRepository + Sync + Send>,
    image_id: String,
) -> Result<(), Box<dyn Error>> {
    image_repository.delete(image_id).await
}
