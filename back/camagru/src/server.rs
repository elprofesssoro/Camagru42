use std::{
    fs,
    io::{prelude::*, BufReader, Error},
    net::{TcpListener, TcpStream},
};

use crate::request::Request;
use crate::routes::routing::route;

pub fn server() -> Result<(), Error> {
    // The ? operator unwraps the Ok value or returns the Err early
    let listener: TcpListener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        handle_connection(stream?)?;
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Error> {
    let mut buf_reader = BufReader::new(&stream);

    let request = match parse_request(&mut buf_reader) {
        Some(request) => request,
        None => {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid request",
            ));
        }
    };

    let response_body = route(&request);

    let status_line: &str = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("log.html")? + &response_body;
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn parse_request(buf_reader: &mut BufReader<&TcpStream>) -> Option<Request> {
    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line).ok()?;

    if request_line.is_empty() {
        return None;
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let version = parts.next()?.to_string();

    let mut content_length = 0;
    let mut content_type: Option<String> = None;

    for line in buf_reader.by_ref().lines() {
        let line = line.ok()?;
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
    buf_reader.read_exact(&mut body_bytes).ok()?;
    let body = if body_bytes.is_empty() {
        None
    } else {
        Some(body_bytes)
    };

    Some(Request {
        method,
        path,
        version,
        body,
        content_length,
        content_type,
    })
}
