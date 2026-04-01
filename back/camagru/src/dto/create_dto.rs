use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePostDTO {
	pub image: String,
	pub sticker_name: String,
	pub pos_x: i64,
    pub pos_y: i64,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HistoryDTO {
	pub img_name: String,
	pub likes: i32,
}