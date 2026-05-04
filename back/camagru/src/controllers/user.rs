use crate::dto::request_dto::{LoginDTO, ReEmailDTO, RePassDTO, RegisterDTO, UserUpdateDTO};
use crate::headers::{Request, Response, Status};
use crate::unwrap_or_return;
use crate::utils::{log_error, send_email, AppState, EmailConfig, extract_json};
use crate::repositories::user_repo::UserRepo;
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use std::sync::OnceLock;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::extract::{Json, State};
use axum_extra::extract::cookie::{Cookie, CookieJar};

pub async fn log_in(State(state): State<Arc<AppState>>, jar: CookieJar, Json(payload): Json<LoginDTO>) -> AxumResponse {
    // let payload = match extract_json::<LoginDTO>(request) {
    //     Ok(payload) => payload,
    //     Err(status) => {
    //         return Response::empty(status);
    //     }
    // };
    let search_by = match validate_login_input(&payload) {
        LoginField::Email => "email",
        LoginField::Username => "username",
        LoginField::Invalid => return StatusCode::BAD_REQUEST.into_response(),
    };

    let db_user = match UserRepo::get_user(&state.db, &payload.cred, search_by).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            log_error("Error fetching user for login", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if db_user.is_deleted || !verify_login(&payload.password, &db_user.password) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    
    if !db_user.is_verified {
        return StatusCode::FORBIDDEN.into_response();
    }
	
	let session = generate_token();
	match UserRepo::session_token_insert(&state.db, &session, db_user.id).await {
		Ok(_) => {
			let mut cookie = Cookie::new("auth_token", session);
			cookie.set_http_only(true);
			cookie.set_path("/");
    		cookie.set_secure(false);
			let updated_jar = jar.add(cookie);
			(updated_jar, StatusCode::OK).into_response()
		},
        Err(e) => {
            log_error("Error saving session token", e);
           StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
	}
}

pub async fn log_out(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = unwrap_or_return!(request.user_id, Status::Unauthorized);

    match UserRepo::delete_session(&state.db, user_id).await {
        Ok(_) => Response::cookie(Status::Ok, String::new()),
        Err(err) => {
            log_error("Database error deleting session token (log_out)", err);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn register(State(state): State<Arc<AppState>>, Json(payload): Json<RegisterDTO>) -> impl IntoResponse {
	if !validate_register_input(&payload) {
        return StatusCode::BAD_REQUEST;
    }
	let hashed = match hash_password(&payload.password) {
        Ok(hashed) => hashed,
        Err(e) => {
            log_error("Error hashing a password", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
	let v_token = generate_token();
	match UserRepo::register_user(&state.db, &payload, &v_token, &hashed).await {
        Ok(_) => {
            let email = payload.email.clone();
            let v_token = v_token.clone();
            let email_conf = state.email_conf.clone();
			
            tokio::spawn(async move {
                prepare_email(email_conf, email, v_token).await;
            });
            StatusCode::OK
        },
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            StatusCode::CONFLICT
        },
        Err(e) => {
            log_error("Error inserting user during registration", e);
            StatusCode::INTERNAL_SERVER_ERROR
        },
    }
//     let payload = match extract_json::<RegisterDTO>(request) {
//         Ok(payload) => payload,
//         Err(status) => return Response::empty(status),
//     };
//     println!("{:?}", payload);
//     if !validate_register_input(&payload) {
//         return Response::empty(Status::BadRequest);
//     }
//     let hashed = match hash_password(&payload.password) {
//         Ok(hashed) => hashed,
//         Err(e) => {
//             log_error("Error hashing a password", e);
//             return Response::empty(Status::InternalServerError);
//         }
//     };

//     let v_token = generate_token();

//     match UserRepo::register_user(&state.db, &payload, &v_token, &hashed).await {
//         Ok(_) => {
//             let email = payload.email.clone();
//             let v_token = v_token.clone();
//             let email_conf = state.email_conf.clone();
//             tokio::spawn(async move {
//                 prepare_email(email_conf, email, v_token).await;
//             });
//             Response::empty(Status::Ok)
//         }
//         Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
//             Response::empty(Status::Conflict)
//         }
//         Err(e) => {
//             log_error("Error inserting user during registration", e);
//             Response::empty(Status::InternalServerError)
//         }
//     }
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

    match UserRepo::verify_user(&state.db, &token).await {
        Ok(true) => {
            Response::redir("/econf".to_string())
		},
		Ok(false) => {
            Response::redir("/error".to_string())
		},
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
    let payload = match extract_json::<ReEmailDTO>(request) {
        Ok(payload) => payload,
        Err(status) => return Response::empty(status),
    };
    if !validate_email(&payload.email) {
        return Response::empty(Status::BadRequest);
    }
    let p_token = generate_token();

    match UserRepo::reset_pass_req(&state.db, &p_token, &payload.email).await {
        Ok(res) => {
			if res.rows_affected() > 0 {
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

pub async fn re_pass_verify(request: &Request) -> Response {
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
    
    match UserRepo::reset_pass_verify(&state.db, &token).await {
        Ok(Some(id)) => {
    		match UserRepo::reset_pass_update(&state.db, &hashed, id).await {
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
    let payload = match extract_json::<ReEmailDTO>(request) {
        Ok(payload) => payload,
        Err(status) => return Response::empty(status),
    };
    let token = generate_token();

    match UserRepo::resend_email(&state.db, &token, &payload.email).await {
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

	match UserRepo::user_info(&state.db, user_id).await {
		Ok(Some(dto)) => {
			match serde_json::to_string(&dto) {
       			Ok(json) => return Response::json(json),
        		Err(e) => {
            		log_error("Error in UserInfo serialization", e);
            		return Response::empty(Status::InternalServerError);
				}
        	};
		},
		Ok(None) => Response::empty(Status::NotFound),
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
	
	let db_password = match UserRepo::get_password(&state.db, user_id).await {
		Ok(password) => password,
		Err(err)  => {
			log_error("Error getting password from DB on update", err);
			return Response::empty(Status::InternalServerError);
		}
	};

	if !verify_login(&payload.current_password, &db_password) {
		return Response::empty(Status::Forbidden);
	}

    if let Some(email) = &payload.email {
		if !validate_email(email) { return Response::empty(Status::BadRequest) }
    }

    if let Some(username) = &payload.username {
		if !validate_username(username) { return Response::empty(Status::BadRequest) }
    }

	let mut hashed_pass = None;
	if let Some(password) = &payload.new_password  {
		if !validate_password(password) { return Response::empty(Status::BadRequest) }
		match hash_password(password) {
        	Ok(hashed) => hashed_pass = Some(hashed),
        	Err(e) => {
        	    log_error("Error hashing a new password", e);
        	    return Response::empty(Status::InternalServerError);
        	}
    	};
	}

    let res = UserRepo::update_user(
		&state.db, 
		user_id,
		payload.email.as_deref(), 
		payload.username.as_deref(),
		payload.notify_comment,
		hashed_pass.as_deref()).await;

    match res {
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

	let db_password = match UserRepo::get_password(&state.db, user_id).await {
		Ok(password) => password,
		Err(err)  => {
			log_error("Error getting password from DB on update", err);
			return Response::empty(Status::InternalServerError);
		}
	};

	if !verify_login(&payload.password, &db_password) {
		return Response::empty(Status::Forbidden);
	}

	match UserRepo::delete_user(&state.db, user_id).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password() {
        assert!(validate_password("Pass1"));
        assert!(validate_password("Secur1ty"));

        assert!(!validate_password("pass"), "Zu kurz");
        assert!(!validate_password("password123"), "Kein Großbuchstabe");
        assert!(!validate_password("PASSWORD123"), "Kein Kleinbuchstabe");
        assert!(!validate_password("Password"), "Keine Zahl");
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("user_123"));
        assert!(validate_username("Valid-Name"));

        assert!(!validate_username("ab"), "Zu kurz");
        assert!(!validate_username("dies_ist_ein_viel_zu_langer_name"), "Zu lang");
        assert!(!validate_username("user@name"), "Ungültiges Sonderzeichen");
    }
}