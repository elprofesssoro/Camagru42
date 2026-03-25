use crate::request::{Request, Response, Status};
use crate::controllers;

pub async fn route(request: &Request) -> Response {
    println!("{:?}", request);
	match request.method.as_str() {
        "GET" => {
            println!("Handling a GET request for path: {}", request.path);
          	routing_get(&request)
        }
        "POST" => {
            println!("Handling a POST request for path: {}", request.path);
            match routing_post(&request) {
                Some(body) => Response::json(body),
                None => Response::empty(Status::NotFound),
            }
        }
        _ => {
            println!("Unknown or unsupported method: {}", request.method);
			Response::empty(Status::BadRequest)
        }
    }
}

fn routing_get(request: &Request) -> Response {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return None;
        }
    };
    let response = match route {
        "login" => Some(controllers::user::log_in(request)),
        _ => None,
    };
    response
}

fn routing_post(request: &Request) -> Option<String> {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return None;
        }
    };

    let response = match route {
        "login" => Some(controllers::user::log_in(request)),
        "register" => Some(controllers::user::register(request)),
        _ => None,
    };
    response
}
