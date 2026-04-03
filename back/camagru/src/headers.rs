#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
	pub query: Option<String>,
	pub cookie: Option<String>,
    pub content_length: usize,
    pub content_type: Option<String>,
    pub version: String,
    pub body: Option<Vec<u8>>,
	pub user_id: Option<i32>,
    pub pub_path: String,
}

#[derive(Debug)]
pub enum Status {
   	Ok,                    // 200
    Created,               // 201
    BadRequest,            // 400
    Unauthorized,		   // 401
    Forbidden,
    NotFound,			   // 404
	UnsupportedMediaType,  // 415
    InternalServerError,   // 500
}

impl Status {
	pub fn code(&self) -> u16 {
		match self {
			Status::Ok => 200,
            Status::Created => 201,
            Status::BadRequest => 400,
            Status::Unauthorized => 401,
            Status::Forbidden => 403,
            Status::NotFound => 404,
			Status::UnsupportedMediaType => 415,
            Status::InternalServerError => 500,
		}
	}
	pub fn message(&self) -> &str {
		match self {
			Status::Ok => "OK",
            Status::Created => "Created",
            Status::BadRequest => "Bad Request",
            Status::Unauthorized => "Unauthorized",
            Status::Forbidden => "Forbidden",
            Status::NotFound => "Not Found",
			Status::UnsupportedMediaType => "Unsupported Media Type",
            Status::InternalServerError => "Internal Server Error",
		}
	}

	pub fn status_line(&self) -> String {
		format!("HTTP/1.1 {} {}", self.code(), self.message())
	}
}

#[derive(Debug)]
pub struct Response {
    pub status: Status,
	pub content_type: Option<String>,
	pub cookie: Option<String>,
    pub body: Option<Vec<u8>>,
}

impl Response {
	fn new(status: Status, content_type: Option<&str>, body: Option<Vec<u8>>, cookie: Option<String>) -> Self {
        Response {
            status: status,
            content_type: content_type.map(|s| s.to_string()),
            body,
			cookie
        }
    }

    pub fn json(body: String) -> Self {
        Response::new(Status::Ok, Some("application/json"), Some(body.into_bytes()), None)
    }

    pub fn html(body: String) -> Self {
        Response::new(Status::Ok, Some("text/html"), Some(body.into_bytes()), None)
    }

	pub fn empty(status: Status) -> Self {
        Response::new(status, None, None, None)
    }

	pub fn cookie(status: Status, cookie: String) -> Self {
        Response::new(status, None, None, Some(cookie))
    }

    pub fn to_http_response(&self) -> Vec<u8> {
        let length = match &self.body {
			Some(b)=> b.len(),
			None => 0,
		};
		let content_type_header = match &self.content_type {
			Some(ct)=> format!("Content-Type: {}\r\n", ct),
			None => String::new(),
		};
		let cookie_header = match &self.cookie {
			Some(cookie) => format!("Set-Cookie: session_id={:?}; HttpOnly; Secure; SameSite=Strict; Max-Age=86400\r\n", cookie),
			None => String::new(),
		};
        let headers = format!(
            "{}\r\n\
        	{}\
            Content-Length: {}\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: GET, POST, OPTIONS, PUT, DELETE\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
			{}\
            \r\n",
            self.status.status_line(), content_type_header, length, cookie_header
        );

		let mut response_bytes = headers.into_bytes();
		if let Some(body_bytes) = &self.body {
			response_bytes.extend(body_bytes);
		}
		response_bytes
	}
}