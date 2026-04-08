use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct GalleryDTO {
	pub author: String,
	pub likes: i64,
	pub img_name: String,
	pub post_id: i32
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PaginatedGalleryDTO {
	pub posts: Vec<GalleryDTO>,
	pub total_posts: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommentDTO {
	pub comment: String,
}