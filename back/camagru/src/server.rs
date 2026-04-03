use std::{env, io::Error, sync::Arc};

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::{TcpListener, TcpStream};

use sqlx::{Connection, PgPool, Row};

use crate::headers::Request;
use crate::routes::routing::route;

pub struct AppState {
    pub name: String,
    pub db: PgPool,
}

pub async fn server() -> Result<(), Error> {
  let mut conn = match connect_db().await {
    Some(conn) => conn,
    None => {
        println!("Database connection was not established");
        return Err(Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Database connection was not established",
        ));
    }
};
    let state = AppState {
        name: String::from("Some Name"),
        db: conn,
    };
    let shared_state = Arc::new(state);
    let listener: TcpListener = TcpListener::bind("0.0.0.0:8080").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let state_thread = Arc::clone(&shared_state);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state_thread).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: TcpStream, state: Arc<AppState>) -> Result<(), Error> {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let request = match parse_request(&mut buf_reader).await {
        Some(request) => request,
        None => {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid request",
            ));
        }
    };

    let response = route(&request, &state).await;

    writer.write_all(&response.to_http_response()).await?;
    writer.flush().await?;
    Ok(())
}

async fn parse_request(buf_reader: &mut BufReader<OwnedReadHalf>) -> Option<Request> {
    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line).await.ok()?;

    if request_line.is_empty() {
        return None;
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let version = parts.next()?.to_string();

    let mut content_length = 0;
    let mut content_type: Option<String> = None;
    let mut cookie: Option<String> = None;

    let mut line = String::new();
    loop {
        line.clear();
        buf_reader.read_line(&mut line).await.ok()?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.to_lowercase().starts_with("content-length:") {
            if let Some((_, value)) = trimmed.split_once(':') {
                content_length = value.trim().parse::<usize>().unwrap_or(0);
            }
        }

        if trimmed.to_lowercase().starts_with("content-type:") {
            if let Some((_, value)) = trimmed.split_once(':') {
                content_type = match value.trim().to_string() {
                    content_type if !content_type.is_empty() => Some(content_type),
                    _ => None,
                };
            }
        }
        if trimmed.to_lowercase().starts_with("cookie:") {
            if let Some((_, value)) = trimmed.split_once(':') {
                cookie = Some(value.trim().to_string());
            }
        }
    }

    let mut body_bytes = vec![0; content_length];
    if content_length > 0 {
        buf_reader.read_exact(&mut body_bytes).await.ok()?;
    }
    let body = if body_bytes.is_empty() {
        None
    } else {
        Some(body_bytes)
    };

    let (path, query) = match path.split_once('?') {
        Some((p, q)) => (p.to_string(), (!q.is_empty()).then(|| q.to_string())),
        None => (path, None),
    };
    let public_dir = env::var("PUBLIC_DIR").unwrap_or_else(|_| "../../pub".to_string());
    Some(Request {
        method,
        path,
        query,
        version,
        body,
        content_length,
        content_type,
        cookie,
        user_id: None,
        pub_path: public_dir,
    })
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
