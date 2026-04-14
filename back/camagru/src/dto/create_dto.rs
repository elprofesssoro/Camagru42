use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePostDTO {
	pub image: String,
	pub sticker_name: String,
	pub pos_x: i64,
    pub pos_y: i64,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct HistoryDTO {
	pub image_path: String,
	pub post_date: DateTime<Utc>,
	pub id: i32
}