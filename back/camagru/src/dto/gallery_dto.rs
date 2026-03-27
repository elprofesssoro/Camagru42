use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct GalleryDTO {
	pub author: String,
	pub likes: i32,
	pub image_path: String,
}