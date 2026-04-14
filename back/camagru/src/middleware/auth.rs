use crate::headers::{Request};
use crate::utils::{AppState, log_error};
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::sync::Arc;

pub async fn auth_middleware(request: &Request, state: &Arc<AppState>) -> Option<i32> {
    let cookie_header = match request.cookie.as_ref() {
        Some(cookie) => cookie,
        None => return None,
    };
    if let Some(session_token) = extract_session_token(&cookie_header) {
        let q = "SELECT user_id, expires_at FROM sessions WHERE session_token = $1";
        let result = sqlx::query(q)
            .bind(&session_token)
            .fetch_optional(&state.db)
            .await;
        match result {
            Ok(Some(row)) => {
                let expires_at: DateTime<Utc> = row.get("expires_at");
                let now = Utc::now();
                if expires_at < now {
                    let q = "DELETE FROM sessions WHERE session_token = $1";
                    let result = sqlx::query(q).bind(&session_token).execute(&state.db).await;
                    match result {
                        Ok(_) => (),
                        Err(err) => {
                            log_error("Database error deleting session token", err);
                        }
                    }
                    return None;
                }

                if expires_at < now + chrono::Duration::days(3) {
                    let q = "UPDATE sessions SET expires_at = NOW() + INTERVAL '7 days' WHERE session_token = $1";
                    let db_clone = state.db.clone();
                    let token_clone = session_token.clone();
                    tokio::spawn(async move {
                        let result = sqlx::query(q).bind(&token_clone).execute(&db_clone).await;
                        match result {
                            Ok(_) => (),
                            Err(err) => {
                                log_error("Database error updating session token", err);
                            }
                        }
                    });
                }

                return Some(row.get("user_id"));
            }
            Ok(None) => return None,
            Err(err) => {
                log_error("Database error in user auth", err);
                return None;
            }
        }
    }

    return None;
}

fn extract_session_token(cookie_header: &str) -> Option<String> {
    for cookie_pair in cookie_header.split(';') {
        let trimmed_pair = cookie_pair.trim();

        if let Some((key, value)) = trimmed_pair.split_once('=') {
            if key == "session_id" {
                return Some(value.to_string());
            }
        }
    }

    None
}
