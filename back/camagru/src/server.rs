use crate::utils::{log_error, AppState, EmailConfig};
use crate::{controllers, middleware as own_middleware};
use axum::middleware;
use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;
use std::{env, io::Error, sync::Arc};
use tokio::net::TcpListener;

pub async fn server() -> Result<(), Error> {
    let conn = match connect_db().await {
        Some(conn) => conn,
        None => {
            println!("Database connection was not established");
            return Err(Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Database connection was not established",
            ));
        }
    };
    let public_dir = env::var("PUBLIC_DIR").unwrap_or_else(|_| "../../pub".to_string());
    let email_conf = match EmailConfig::get_env() {
        Ok(conf) => conf,
        Err(err) => {
            log_error("Error parsing email configuration", err);
            return Err(Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Error parsing email configuration",
            ));
        }
    };
    let state = AppState {
        db: conn,
        email_conf,
        img_root_dir: public_dir,
    };
    let shared_state = Arc::new(state);
    let listener: TcpListener = TcpListener::bind("0.0.0.0:8080").await?;
    let app = Box::new(Router::new())
        .route("/api/logout", post(controllers::user::log_out))
        .route("/api/me", get(controllers::user::me))
        .route("/api/gallery/like", post(controllers::gallery::like))
        .route("/api/gallery/comment", post(controllers::gallery::comment))
        .route("/api/create/history", get(controllers::create::create_get))
        .route("/api/create/post", post(controllers::create::create_post))
        .route(
            "/api/create/delete",
            delete(controllers::create::create_delete),
        )
        .route(
            "/api/create/details",
            get(controllers::create::create_details),
        )
        .route("/api/user/info", get(controllers::user::info))
        .route("/api/user/update", post(controllers::user::update))
        .route("/api/user/delete", delete(controllers::user::delete))
        .route_layer(middleware::from_fn_with_state(
            shared_state.clone(),
            own_middleware::auth::auth_middleware,
        ))
        .route("/api/login", post(controllers::user::log_in))
        .route("/api/register", post(controllers::user::register))
        .route("/api/gallery", get(controllers::gallery::gallery))
        .route("/api/verify", get(controllers::user::user_verify))
        .route(
            "/api/re-pass/verify",
            get(controllers::user::re_pass_verify),
        )
        .route("/api/re-pass", post(controllers::user::re_pass))
        .route("/api/re-pass/new", post(controllers::user::re_pass_new))
        .route("/api/re-email", post(controllers::user::re_email))
        .with_state(shared_state);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn connect_db() -> Option<PgPool> {
    let user = env::var("DB_USER").ok()?;
    let db = env::var("DB_NAME").ok()?;
    let pass = env::var("DB_PASSWORD").ok()?;
    let host = env::var("DB_HOST").ok()?;
    let port = env::var("DB_PORT").ok()?;
    let url = format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, db);
    Some(sqlx::postgres::PgPool::connect(&url).await.ok()?)
}
