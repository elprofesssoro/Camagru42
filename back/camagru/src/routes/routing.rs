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
            routing_post(&request)
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
            return Response::empty(Status::NotFound);
        }
    };
    let response = match route {
        "login" => Response::json(controllers::user::log_in(request)),
        _ => Response::empty(Status::NotFound),
    };
    response
}

fn routing_post(request: &Request) -> Response {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return Response::empty(Status::NotFound);
        }
    };

    let response = match route {
        "login" => {
			controllers::user::log_in(request);
			Response::empty(Status::Ok)
		},
        "register" => {
			controllers::user::register(request);
			Response::empty(Status::Ok)
		},
        _ =>  {
			Response::empty(Status::NotFound)
		},
    };
    response
}
