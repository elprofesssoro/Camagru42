use std::{
    io::Error,
	sync::Arc
};

use tokio::net::tcp::OwnedReadHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

use crate::headers::{Response, Request, Status};
use crate::routes::routing::route;

pub struct AppState {
	pub name: String,
}

pub async fn server() -> Result<(), Error> {
	let state = AppState {
		name: String::from("Some Name")
	};
	let shared_state = Arc::new(state);
    let listener: TcpListener = TcpListener::bind("127.0.0.1:8080").await?;

	loop {
		let (stream, _) = listener.accept().await?;
		let state_thread = Arc::clone(&shared_state);
		tokio::spawn(async move{
			if let Err(e) = handle_connection(stream, state_thread).await {
				eprintln!("Error handling connection: {}", e);
			}
		});
	}

    Ok(())
}

async fn handle_connection(mut stream: TcpStream, state: Arc<AppState>) -> Result<(), Error> {
	let(reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
	println!("{}", state.name);
    let request = match parse_request(&mut buf_reader).await {
        Some(request) => request,
        None => {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid request",
            ));
        }
    };

    let response = route(&request).await;

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

	let mut line = String::new();
    loop {
		line.clear();
		buf_reader.read_line(&mut line).await.ok()?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.to_lowercase().starts_with("content-length:") {
            if let Some(value) = trimmed.split(':').nth(1) {
                content_length = value.trim().parse::<usize>().unwrap_or(0);
            }
        }

        if trimmed.to_lowercase().starts_with("content-type:") {
            if let Some(value) = trimmed.split(':').nth(1) {
                content_type = match value.trim().to_string() {
                    content_type if !content_type.is_empty() => Some(content_type),
                    _ => None,
                };
            }
        }
    }

    let mut body_bytes = vec![0; content_length];
	if (content_length > 0) {
	    buf_reader.read_exact(&mut body_bytes).await.ok()?;
	}
    let body = if body_bytes.is_empty() { None } else { Some(body_bytes) };

    Some(Request {
        method,
        path,
        version,
        body,
        content_length,
        content_type,
    })
}
