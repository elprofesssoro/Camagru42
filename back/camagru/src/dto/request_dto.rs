use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginDTO {
	pub cred: String,
	pub password: String,
}


#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterDTO {
	pub email: String,
	pub username: String,
	pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReEmailDTO {
	pub email: String
}

#[derive(Deserialize, Debug)]
pub struct RePassDTO {
	pub password: String
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct UserInfoDTO {
	pub email: String,
	pub username: String,
	pub notify_comment: bool
}

#[derive(Deserialize, Debug)]
pub struct UserUpdateDTO {
	pub email: Option<String>,
	pub username: Option<String>,
	pub notify_comment: Option<bool>,
	pub new_password: Option<String>,
	pub current_password: String
}