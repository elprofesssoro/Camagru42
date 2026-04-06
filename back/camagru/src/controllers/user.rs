use crate::dto::request_dto::{LoginDTO, RegisterDTO};
use crate::headers::{log_error, Request, Response, Status};
use crate::server::AppState;
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::from_slice;
use sqlx::Row;
use std::sync::Arc;

pub async fn log_in_get(request: &Request, state: &Arc<AppState>) -> Response {
    let content_type = request.content_type.as_deref().unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Response::empty(Status::UnsupportedMediaType);
    }

    let body = match request.body.as_ref() {
        Some(body) => body,
        None => return Response::empty(Status::BadRequest),
    };

    let payload = match from_slice::<LoginDTO>(body) {
        Ok(payload) => payload,
        Err(_) => return Response::empty(Status::BadRequest),
    };
    println!("{:?}", payload);
    let valid = validate_login_input(&payload);
    if valid == 0 {
        return Response::empty(Status::BadRequest);
    }
    let search_by = match valid {
        1 => "email",
        2 => "username",
        _ => return Response::empty(Status::BadRequest),
    };
    Response::empty(Status::Ok)
}

pub async fn log_in_post(request: &Request, state: &Arc<AppState>) -> Response {
    let content_type = request.content_type.as_deref().unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Response::empty(Status::UnsupportedMediaType);
    }

    let body = match request.body.as_ref() {
        Some(body) => body,
        None => return Response::empty(Status::BadRequest),
    };

    let payload = match from_slice::<LoginDTO>(body) {
        Ok(payload) => payload,
        Err(err) => {
            log_error("Error deserializing slice", err);
            return Response::empty(Status::NotFound);
        }
    };
    println!("{:?}", payload);
    let valid = validate_login_input(&payload);
    if valid == 0 {
        return Response::empty(Status::BadRequest);
    }
    let search_by = match valid {
        1 => "email",
        2 => "username",
        _ => return Response::empty(Status::BadRequest),
    };
    let q = format!(
        "SELECT password, is_verified FROM users WHERE {} = $1",
        search_by
    );

    let result = sqlx::query(&q)
        .bind(&payload.cred)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(row)) => {
            let db_password: String = row.get("password");
            let is_verified: bool = row.get("is_verified");

            //TODO: Password hash verification
            if db_password != payload.password {
                return Response::empty(Status::Unauthorized);
            }
            println!("DB WAS RECEIVED: {}, {}", db_password, is_verified);
            if !is_verified {
                return Response::empty(Status::Forbidden);
            }
            Response::cookie(Status::Ok, generate_session_token())
        }
        Ok(None) => {
            println!("User not found.");
            Response::empty(Status::Unauthorized)
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn register(request: &Request, state: &Arc<AppState>) -> Response {
    let content_type = request.content_type.as_deref().unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Response::empty(Status::UnsupportedMediaType);
    }

    let body = match request.body.as_ref() {
        Some(body) => body,
        None => return Response::empty(Status::BadRequest),
    };
    let payload = match from_slice::<RegisterDTO>(body) {
        Ok(payload) => payload,
        Err(_) => return Response::empty(Status::BadRequest),
    };
    println!("{:?}", payload);
    if !validate_register_input(&payload) {
        return Response::empty(Status::BadRequest);
    }
    let hashed = match hash_password(&payload.password) {
        Ok(hashed) => hashed,
        Err(e) => {
            log_error("Error hashing a password", e);
            return Response::empty(Status::InternalServerError);
        }
    };

    let q = "INSERT INTO users (email, username, password) VALUES ($1, $2, $3)";
    let result = sqlx::query(&q)
        .bind(&payload.email)
        .bind(&payload.username)
        .bind(&hashed)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => (),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return Response::empty(Status::Conflict);
        },
        Err(e) => {
            log_error("Error inserting user during registration", e);
            return Response::empty(Status::InternalServerError);
        }
    }
    Response::empty(Status::Ok)
}

fn validate_login_input(login_dto: &LoginDTO) -> u8 {
    if validate_password(login_dto.password.as_str()) {
        if validate_email(login_dto.cred.as_str()) {
            return 1;
        } else if validate_username(login_dto.cred.as_str()) {
            return 2;
        }
    }
    return 0;
}

fn validate_register_input(register_dto: &RegisterDTO) -> bool {
    validate_email(register_dto.email.as_str())
        && validate_username(register_dto.username.as_str())
        && validate_password(register_dto.password.as_str())
}

fn validate_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    email_regex.is_match(email)
}

fn validate_username(username: &str) -> bool {
    let is_valid_length = username.len() >= 3 && username.len() <= 20;
    let is_alphanumeric = username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-');
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

fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

fn generate_session_token() -> String {
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    token
}
