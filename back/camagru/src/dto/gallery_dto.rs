use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct GalleryDTO {
	pub author: String,
	pub likes: usize,
	pub img_name: String,
	pub post_id: usize
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