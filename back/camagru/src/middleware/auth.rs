use crate::utils::{log_error, AppState};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::sync::Arc;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_token = match jar.get("auth_token") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(StatusCode::UNAUTHORIZED),
    };

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
                let _ = sqlx::query(q).bind(&session_token).execute(&state.db).await;
                return Err(StatusCode::UNAUTHORIZED);
            }

            if expires_at < now + chrono::Duration::days(3) {
                let q = "UPDATE sessions SET expires_at = NOW() + INTERVAL '7 days' WHERE session_token = $1";
                let db_clone = state.db.clone();
                let token_clone = session_token.clone();
                tokio::spawn(async move {
                    if let Err(err) = sqlx::query(q).bind(&token_clone).execute(&db_clone).await {
                        log_error("Database error updating session token", err);
                    }
                });
            }

            let user_id: i32 = row.get("user_id");
            request.extensions_mut().insert(user_id);
            Ok(next.run(request).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(err) => {
            log_error("Database error in user auth", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
