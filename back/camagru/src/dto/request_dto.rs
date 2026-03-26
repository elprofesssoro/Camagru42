use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginDTO {
	pub cred: String,
	pub password: String,
}


#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterDTO {
	pub email: String,
	pub password: String,
	pub username: String
}