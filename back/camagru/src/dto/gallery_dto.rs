use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct GalleryDTO {
	pub author: String,
	pub likes: usize,
	pub image_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PaginatedGalleryDTO {
	pub posts: Vec<GalleryDTO>,
	pub total_posts: usize,
}