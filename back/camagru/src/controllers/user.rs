use crate::dto::request_dto::{LoginDTO, ReEmailDTO, RePassDTO, RegisterDTO, UserInfoDTO, UserUpdateDTO};
use crate::headers::{Request, Response, Status};
use crate::unwrap_or_return;
use crate::utils::{log_error, send_email, AppState, EmailConfig, extract_json};
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::from_slice;
use sqlx::Row;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use std::sync::OnceLock;

pub async fn log_in_post(request: &Request, state: &Arc<AppState>) -> Response {
    // let content_type = request.content_type.as_deref().unwrap_or("");

    // if !content_type.starts_with("application/json") {
    //     return Response::empty(Status::UnsupportedMediaType);
    // }

    // let body = unwrap_or_return!(request.body.as_ref(), Status::BadRequest);
    // // let body = match request.body.as_ref() {
    // //     Some(body) => body,
    // //     None => return Response::empty(Status::BadRequest),
    // // };

    let payload = match extract_json::<LoginDTO>(request) {
        Ok(payload) => payload,
        Err(status) => {
            return Response::empty(status);
        }
    };
    let search_by = match validate_login_input(&payload) {
        LoginField::Email => "email",
        LoginField::Username => "username",
        LoginField::Invalid => return Response::empty(Status::BadRequest),
    };
    let q = format!(
        "SELECT id, password, is_verified, is_deleted FROM users WHERE {} = $1",
        search_by
    );

    let result = sqlx::query(&q)
        .bind(&payload.cred)
        .fetch_optional(&state.db)
        .await;

    let (session, user_id) = match result {
        Ok(Some(row)) => {
            let db_password: String = row.get("password");
            let is_verified: bool = row.get("is_verified");
			let is_deleted: bool = row.get("is_deleted");
			if is_deleted {
				return Response::empty(Status::Unauthorized)
			}
            if !verify_login(&payload.password, &db_password) {
                return Response::empty(Status::Unauthorized);
            }
            println!("DB WAS RECEIVED: {}, {}", db_password, is_verified);
            if !is_verified {
                return Response::empty(Status::Forbidden);
            }
            let id: i32 = row.get("id");
            (generate_token(), id)
        }
        Ok(None) => {
            return Response::empty(Status::Unauthorized);
        }
        Err(e) => {
            log_error("Error saving user", e);
            return Response::empty(Status::InternalServerError);
        }
    };

    session_token_insert(state, session, user_id).await
}

pub async fn log_out(request: &Request, state: &Arc<AppState>) -> Response {
    if request.user_id == None {
        return Response::empty(Status::Unauthorized);
    }
    let q = "DELETE FROM sessions WHERE user_id = $1";
    let result = sqlx::query(q)
        .bind(&request.user_id)
        .execute(&state.db)
        .await;
    match result {
        Ok(_) => Response::cookie(Status::Ok, String::new()),
        Err(err) => {
            log_error("Database error deleting session token (log_out)", err);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn register(request: &Request, state: &Arc<AppState>) -> Response {
    // let content_type = request.content_type.as_deref().unwrap_or("");

    // if !content_type.starts_with("application/json") {
    //     return Response::empty(Status::UnsupportedMediaType);
    // }

    // let body = match request.body.as_ref() {
    //     Some(body) => body,
    //     None => return Response::empty(Status::BadRequest),
    // };
    let payload = match extract_json::<RegisterDTO>(request) {
        Ok(payload) => payload,
        Err(status) => return Response::empty(status),
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
    let v_token = generate_token();
    let q =
        "INSERT INTO users (email, username, password, verification_token) VALUES ($1, $2, $3, $4)";
    let result = sqlx::query(&q)
        .bind(&payload.email)
        .bind(&payload.username)
        .bind(&hashed)
        .bind(&v_token)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            let email = payload.email.clone();
            let v_token = v_token.clone();
            let email_conf = state.email_conf.clone();
            tokio::spawn(async move {
                prepare_email(email_conf, email, v_token).await;
            });
            Response::empty(Status::Ok)
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Response::empty(Status::Conflict)
        }
        Err(e) => {
            log_error("Error inserting user during registration", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn user_verify(request: &Request, state: &Arc<AppState>) -> Response {
    let query = match &request.query {
        Some(query) => query,
        None => return Response::redir("/error".to_string()),
    };
    let token = match token_query(&query) {
        Some(token) => token,
        None => return Response::redir("/error".to_string()),
    };
    let q = "UPDATE users SET is_verified = TRUE, verification_token = NULL WHERE verification_token = $1";
    let result = sqlx::query(&q).bind(&token).execute(&state.db).await;
    match result {
        Ok(rows) => {
            if rows.rows_affected() == 1 {
                return Response::redir("/econf".to_string());
            } else {
                Response::redir("/error".to_string())
            }
        }
        Err(e) => {
            log_error("Database error verifying user", e);
            Response::redir("/error".to_string())
        }
    }
}

pub async fn me(request: &Request) -> Response {
    if request.user_id == None {
        Response::empty(Status::Unauthorized)
    } else {
        Response::empty(Status::Ok)
    }
}

pub async fn re_pass(request: &Request, state: &Arc<AppState>) -> Response {
	// let content_type = request.content_type.as_deref().unwrap_or("");

    // if !content_type.starts_with("application/json") {
    //     return Response::empty(Status::UnsupportedMediaType);
    // }

	// let body = unwrap_or_return!(request.body.as_ref(), Status::BadRequest);
    // // let body = match request.body.as_ref() {
    // //     Some(body) => body,
    // //     None => return Response::empty(Status::BadRequest),
    // // };
    let payload = match extract_json::<ReEmailDTO>(request) {
        Ok(payload) => payload,
        Err(status) => return Response::empty(status),
    };
    if !validate_email(&payload.email) {
        return Response::empty(Status::BadRequest);
    }
    let p_token = generate_token();
    let q =
        "UPDATE users SET reset_verification_token = $1, reset_expires_at = NOW() + INTERVAL '5 minutes' WHERE email = $2";
    let result = sqlx::query(&q)
        .bind(&p_token)
        .bind(&payload.email)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) => {
			if (res.rows_affected() > 0){
				let email_conf = state.email_conf.clone();
            	tokio::spawn(async move {
            	    prepare_reset_email(email_conf, payload.email, p_token).await;
            	});
			}
            Response::empty(Status::Ok)
        }
        Err(e) => {
            log_error("Error updating reset token", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn re_pass_verify(request: &Request, state: &Arc<AppState>) -> Response {
	let query = match request.query.as_ref() {
        Some(q) => q,
        None => return Response::redir("/error".to_string()),
    };

    let token = match token_query(query) {
        Some(t) => t,
        None => return Response::redir("/error".to_string()),
    };

    let frontend_url = format!("/resetPass?token={}", token);
    Response::redir(frontend_url)
}

pub async fn re_pass_new(request: &Request, state: &Arc<AppState>) -> Response {
	// let content_type = request.content_type.as_deref().unwrap_or("");

    // if !content_type.starts_with("application/json") {
    //     return Response::empty(Status::UnsupportedMediaType);
    // }
	// let body = unwrap_or_return!(request.body.as_ref(), Status::BadRequest);
    // let payload = match from_slice::<RePassDTO>(body) {
    //     Ok(payload) => payload,
    //     Err(_) => return Response::empty(Status::BadRequest),
    // };
	let payload = match extract_json::<RePassDTO>(request) {
 		Ok(payload) => payload,
        Err(status) => return Response::empty(status),
	};
	let query = unwrap_or_return!(&request.query, Status::BadRequest);
	let token = unwrap_or_return!(token_query(&query), Status::BadRequest);
    let hashed = match hash_password(&payload.password) {
        Ok(hashed) => hashed,
        Err(e) => {
            log_error("Error hashing a password", e);
            return Response::empty(Status::InternalServerError);
        }
    };
    let q =
        "SELECT id FROM users WHERE reset_verification_token = $1 AND reset_expires_at > NOW()";
    let result = sqlx::query(&q)
        .bind(token)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(row)) => {
			let id: i32 = row.get("id");
            let q =
      		  	"UPDATE users SET password = $1, reset_verification_token = NULL, reset_expires_at = NULL WHERE id = $2";
    		let result = sqlx::query(&q)
    		    .bind(&hashed)
				.bind(id)
    		    .execute(&state.db)
    		    .await;

    		match result {
    		    Ok(_) => {
    		        Response::empty(Status::Ok)
    		    }
    		    Err(e) => {
    		        log_error("Error updating reset token", e);
    		        Response::empty(Status::InternalServerError)
    		    }
    		}
        },
		Ok(None) => {
			Response::empty(Status::BadRequest)
		},
        Err(e) => {
            log_error("Error updating reset token", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn re_email(request: &Request, state: &Arc<AppState>) -> Response {
    // let content_type = request.content_type.as_deref().unwrap_or("");
    // if !content_type.starts_with("application/json") {
    //     return Response::empty(Status::UnsupportedMediaType);
    // }

    // let body = match request.body.as_ref() {
    //     Some(body) => body,
    //     None => return Response::empty(Status::BadRequest),
    // };
    let payload = match extract_json::<ReEmailDTO>(request) {
        Ok(payload) => payload,
        Err(status) => return Response::empty(status),
    };
    let token = generate_token();
    let q = "UPDATE users 
		SET verification_token = $1 
		WHERE email = $2 AND is_verified = FALSE";
    let result = sqlx::query(&q)
        .bind(&token)
        .bind(&payload.email)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return Response::empty(Status::Ok);
            }

            let email = payload.email.clone();
            let token = token.clone();
            let email_conf_clone = state.email_conf.clone();

            tokio::spawn(async move {
                prepare_email(email_conf_clone, email, token).await;
            });
            Response::empty(Status::Ok)
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Response::empty(Status::Conflict)
        }
        Err(e) => {
            log_error("Error inserting user during registration", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn info(request: &Request, state: &Arc<AppState>) -> Response {
	 let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

	let q = "SELECT email, username, notify_comment FROM users WHERE id = $1";
	let res = sqlx::query_as::<_, UserInfoDTO>(q)
		.bind(user_id)
		.fetch_one(&state.db)
		.await;
	match res {
		Ok(dto) => {
			match serde_json::to_string(&dto) {
       			Ok(json) => return Response::json(json),
        		Err(e) => {
            		log_error("Error in UserInfo serialization", e);
            		return Response::empty(Status::InternalServerError);
				}
        	};
		},
		Err(sqlx::Error::RowNotFound) => Response::empty(Status::NotFound),
        Err(e) => {
            log_error("Database error fetching user info details", e);
            Response::empty(Status::InternalServerError)
        }
	}
}

pub async fn update(request: &Request, state: &Arc<AppState>) -> Response {
	let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

	let payload = match extract_json::<UserUpdateDTO>(request) {
        Ok(payload) => payload,
        Err(status) => return Response::empty(status),
    };

	if payload.email.is_none() 
        && payload.username.is_none() 
        && payload.notify_comment.is_none() 
        && payload.new_password.is_none() {
        return Response::empty(Status::BadRequest);
    }
	
	let q = "SELECT password FROM users WHERE id = $1";
	let res = sqlx::query(q)
		.bind(user_id)
		.fetch_one(&state.db)
		.await;

	let db_password = match res {
		Ok(row) => row.get::<String, _>("password"),
		Err(err)  => {
			log_error("Error getting password from DB on update", err);
			return Response::empty(Status::InternalServerError);
		}
	};

	if !verify_login(&payload.current_password, &db_password) {
		return Response::empty(Status::Forbidden);
	}

    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("UPDATE users SET ");
	let mut separated = query_builder.separated(", ");

    if let Some(email) = &payload.email {
		if (!validate_email(email)) { return Response::empty(Status::BadRequest) }
        separated.push("email = ").push_bind_unseparated(email);
    }

    if let Some(username) = &payload.username {
		if (!validate_username(username)) { return Response::empty(Status::BadRequest) }
        separated.push("username = ").push_bind_unseparated(username);
    }

    if let Some(notify) = &payload.notify_comment {
        separated.push("notify_comment = ").push_bind_unseparated(notify);
    }

	if let Some(password) = &payload.new_password  {
		if (!validate_password(password)) { return Response::empty(Status::BadRequest) }
		let new_hashed = match hash_password(password) {
        	Ok(hashed) => hashed,
        	Err(e) => {
        	    log_error("Error hashing a new password", e);
        	    return Response::empty(Status::InternalServerError);
        	}
    	};
        separated.push("password = ").push_bind_unseparated(new_hashed);
	}

    query_builder.push(" WHERE id = ");
    query_builder.push_bind(user_id);

    let query = query_builder.build();
    match query.execute(&state.db).await {
        Ok(_) => Response::empty(Status::Ok),
        Err(e) => {
            log_error("Database error updating user", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn delete(request: &Request, state: &Arc<AppState>) -> Response {
	let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

	let payload = match extract_json::<RePassDTO>(request) {
		Ok(payload) => payload,
		Err(status) => return Response::empty(status)
	};

	let q = "SELECT password FROM users WHERE id = $1";
	let res = sqlx::query(q)
		.bind(user_id)
		.fetch_one(&state.db)
		.await;

	let db_password = match res {
		Ok(row) => row.get::<String, _>("password"),
		Err(err)  => {
			log_error("Error getting password from DB on update", err);
			return Response::empty(Status::InternalServerError);
		}
	};

	if !verify_login(&payload.password, &db_password) {
		return Response::empty(Status::Forbidden);
	}

	let q = "UPDATE users 
    SET is_deleted = TRUE, 
        email = 'deleteduser',
        username = 'deleteduser',
        password = 'deleted'
    WHERE id = $1";
	let res = sqlx::query(q)
		.bind(user_id)
		.execute(&state.db)
		.await;

	match res {
		Ok(_) => Response::cookie(Status::Ok, String::new()),
		Err(err) => {
			log_error("Error deleting user", err);
			Response::empty(Status::InternalServerError)
		}
	}
}

async fn prepare_email(email_conf: EmailConfig, recv_email: String, token: String) {
    let verify_link = format!("http://localhost:80/api/verify?token={}", token);
    let from = format!("Camagru Admin <{}>", email_conf.get_email());
    let to = format!("<{}>", recv_email);
    let subject = "Welcome to Camagru! Verify your account".to_string();
    let body = format!(
        "Please click the following link to verify your account: {}",
        verify_link
    );
    send_email(email_conf, from, to, subject, body).await
}

async fn prepare_reset_email(email_conf: EmailConfig, recv_email: String, token: String) {
    let reset_link = format!("http://localhost:80/api/re-pass/verify?token={}", token);
    let from = format!("Camagru Admin <{}>", email_conf.get_email());
    let to = format!("<{}>", recv_email);
    let subject = "Reset your password".to_string();
    let body = format!(
        "Please click the following link to reset your password: {}",
        reset_link
    );
    send_email(email_conf, from, to, subject, body).await
}

enum LoginField {
	Email,
	Username,
	Invalid
}
fn validate_login_input(login_dto: &LoginDTO) -> LoginField {
    if validate_password(login_dto.password.as_str()) {
        if validate_email(login_dto.cred.as_str()) {
            return LoginField::Email;
        } else if validate_username(login_dto.cred.as_str()) {
            return LoginField::Username;
        }
    }
    return LoginField::Invalid;
}

fn validate_register_input(register_dto: &RegisterDTO) -> bool {
    validate_email(register_dto.email.as_str())
        && validate_username(register_dto.username.as_str())
        && validate_password(register_dto.password.as_str())
}

fn validate_email(email: &str) -> bool {
	static EMAIL_REGEX: OnceLock<regex::Regex> = OnceLock::new();
	let regex = EMAIL_REGEX.get_or_init(|| {
        regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap()
    });
    
    regex.is_match(email)
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

fn verify_login(password: &str, hash: &str) -> bool {
    println!("Verifying: {} against {}", password, hash);
    match verify(password, hash) {
        Ok(is_valid) => is_valid,
        Err(_) => false,
    }
}

fn generate_token() -> String {
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    token
}

async fn session_token_insert(state: &Arc<AppState>, session: String, user_id: i32) -> Response {
    let q = "INSERT INTO sessions (session_token, user_id, expires_at) 
        VALUES ($1, $2, NOW() + INTERVAL '5 minutes')
        ON CONFLICT (user_id) 
        DO UPDATE SET 
            session_token = EXCLUDED.session_token, 
            expires_at = EXCLUDED.expires_at";

    let result = sqlx::query(q)
        .bind(&session)
        .bind(&user_id)
        .execute(&state.db)
        .await;
    match result {
        Ok(_) => Response::cookie(Status::Ok, session),
        Err(e) => {
            log_error("Error saving session token", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub fn token_query(query: &str) -> Option<String> {
    let mut key_value = query.splitn(2, '=');
    let key = key_value.next().unwrap_or("");
    let value = key_value.next().unwrap_or("");

    match key {
        "token" => Some(value.to_string()),
        _ => return None,
    }
}

// fn parse_env() -> (String, String) {
//     let username = match env::var("EMAIL_HOST") {
//         Ok(str) => str,
//         Err(_) => "default@gmail.com".to_string(),
//     };
//     let password = match env::var("PASSWORD_HOST") {
//         Ok(str) => str,
//         Err(_) => "123345".to_string(),
//     };
//     (username, password)
// }
