use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePostDTO {
	pub image: String,
	pub sticker_path: String,
	pub pos_x: i64,
    pub pos_y: i64,
    pub width: u32,
    pub height: u32,
}