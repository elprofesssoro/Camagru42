use crate::headers::{Request, Response, Status};
use crate::dto::request_dto::{LoginDTO, RegisterDTO};
use serde_json::from_slice;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub async fn log_in_get(request: &Request) -> Response{
    let content_type = request.content_type.as_deref().unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Response::empty(Status::UnsupportedMediaType);
    }

	let body = match request.body.as_ref() {
		Some(body) => body,
		None => return Response::empty(Status::BadRequest)
	};

	let res = match from_slice::<LoginDTO>(body) {
		Ok(res) => res,
		Err(_) => return Response::empty(Status::BadRequest)
	};
    println!("{:?}", res);
	if !validate_login_input(&res) {
		return Response::empty(Status::BadRequest);
	}
    Response::empty(Status::Ok)
}

pub async fn log_in_post(request: &Request) -> Response{
			println!("2OOK");

    let content_type = request.content_type.as_deref().unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Response::empty(Status::UnsupportedMediaType);
    }

	let body = match request.body.as_ref() {
		Some(body) => body,
		None => return Response::empty(Status::BadRequest)
	};

	let res = match from_slice::<LoginDTO>(body) {
		Ok(res) => res,
		Err(_) => return Response::empty(Status::NotFound)
	};
    println!("{:?}", res);
	if !validate_login_input(&res) {
		return Response::empty(Status::BadRequest);
	}
    Response::cookie(Status::Ok, generate_session_token())
}

pub async fn register(request: &Request) -> Response{
				println!("3OOK");

    let content_type = request.content_type.as_deref().unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Response::empty(Status::UnsupportedMediaType);
    }

	let body = match request.body.as_ref() {
		Some(body) => body,
		None => return Response::empty(Status::BadRequest)
	};
	let res = match from_slice::<RegisterDTO>(body) {
		Ok(res) => res,
		Err(_) => return Response::empty(Status::BadRequest)
	};
    println!("{:?}", res);
	if !validate_register_input(&res) {
		return Response::empty(Status::BadRequest);
	}
    Response::empty(Status::Ok)
}


fn validate_login_input(loginDto: &LoginDTO) -> bool {
	if validate_password(loginDto.password.as_str()) {
		return validate_email(loginDto.cred.as_str()) || validate_username(loginDto.cred.as_str());
	}
	false
}

fn validate_register_input(registerDto: &RegisterDTO) -> bool {
	validate_email(registerDto.email.as_str())
		&& validate_username(registerDto.username.as_str())
		&& validate_password(registerDto.password.as_str())
}

fn validate_email(email: &str) -> bool {
	let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
	email_regex.is_match(email)
}

fn validate_username(username: &str) -> bool {
	let is_valid_length = username.len() >= 3 && username.len() <= 20;
	let is_alphanumeric = username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-');
	is_valid_length && is_alphanumeric
}

fn validate_password(password: &str) -> bool {
	if password.len() < 5 {
		return false;
	}
	let has_uppercase = password.chars().any(|c| c.is_uppercase());
	let has_lowercase = password.chars().any(|c| c.is_lowercase());
	let has_digit = password.chars().any(|c| c.is_ascii_digit());
	has_uppercase && has_lowercase && has_digit
}

fn generate_session_token() -> String {
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
        
    token
}