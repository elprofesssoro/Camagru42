
#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub cookie: Option<String>,
    pub content_length: usize,
    pub content_type: Option<String>,
    pub _version: String,
    pub body: Option<Vec<u8>>,
    pub user_id: Option<i32>,
    pub pub_path: String,
}

#[derive(Debug)]
pub enum Status {
    Ok,                   // 200
    Created,              // 201
    SeeOther,             // 303
    BadRequest,           // 400
    Unauthorized,         // 401
    Forbidden,            // 403
    NotFound,             // 404
    Conflict,             // 409
    UnsupportedMediaType, // 415
    InternalServerError,  // 500
}

impl Status {
    pub fn code(&self) -> u16 {
        match self {
            Status::Ok => 200,
            Status::Created => 201,
            Status::SeeOther => 303,
            Status::BadRequest => 400,
            Status::Unauthorized => 401,
            Status::Forbidden => 403,
            Status::NotFound => 404,
            Status::Conflict => 409,
            Status::UnsupportedMediaType => 415,
            Status::InternalServerError => 500,
        }
    }
    pub fn message(&self) -> &str {
        match self {
            Status::Ok => "OK",
            Status::Created => "Created",
            Status::SeeOther => "See Other",
            Status::BadRequest => "Bad Request",
            Status::Unauthorized => "Unauthorized",
            Status::Forbidden => "Forbidden",
            Status::NotFound => "Not Found",
            Status::Conflict => "Conflict",
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
    pub location: Option<String>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    fn new(
        status: Status,
        content_type: Option<&str>,
        body: Option<Vec<u8>>,
        cookie: Option<String>,
        location: Option<String>,
    ) -> Self {
        Response {
            status: status,
            content_type: content_type.map(|s| s.to_string()),
            body,
            cookie,
            location,
        }
    }

    pub fn json(body: String) -> Self {
        Response::new(
            Status::Ok,
            Some("application/json"),
            Some(body.into_bytes()),
            None,
            None,
        )
    }

    // pub fn html(body: String) -> Self {
    //     Response::new(Status::Ok, Some("text/html"), Some(body.into_bytes()), None, None)
    // }

    pub fn empty(status: Status) -> Self {
        Response::new(status, None, None, None, None)
    }

    pub fn cookie(status: Status, cookie: String) -> Self {
        Response::new(status, None, None, Some(cookie), None)
    }

    pub fn redir(location: String) -> Self {
        Response::new(Status::SeeOther, None, None, None, Some(location))
    }

    pub fn to_http_response(&self) -> Vec<u8> {
        let length = match &self.body {
            Some(b) => b.len(),
            None => 0,
        };
        let content_type_header = match &self.content_type {
            Some(ct) => format!("Content-Type: {}\r\n", ct),
            None => String::new(),
        };
        let cookie_header = match &self.cookie {
            Some(cookie) => {
                let age: String = if cookie == "" {
                    format!("0")
                } else {
                    format!("300")
                };
                format!("Set-Cookie: session_id={}; HttpOnly; Path=/; SameSite=Lax; Max-Age={}\r\n", cookie, age)
            }
            None => String::new(),
        };
        let location_header = match &self.location {
            Some(location) => format!("Location: {}\r\n", location),
            None => String::new(),
        };
        let headers = format!(
            "{}\r\n\
        	{}\
            Content-Length: {}\r\n\
            Access-Control-Allow-Origin: http://localhost\r\n\
            Access-Control-Allow-Credentials: true\r\n\
            Access-Control-Allow-Methods: GET, POST, OPTIONS, PUT, DELETE\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
			{}\
            {}\
            \r\n",
            self.status.status_line(),
            content_type_header,
            length,
            cookie_header,
            location_header,
        );

        let mut response_bytes = headers.into_bytes();
        if let Some(body_bytes) = &self.body {
            response_bytes.extend(body_bytes);
        }
        response_bytes
    }
}