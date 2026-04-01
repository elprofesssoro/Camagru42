use crate::headers::{Request, Response, Status};
use crate::controllers;

pub async fn route(request: &Request) -> Response {
	if (request.content_length < 100) { println!("{:?}", request); }
	match request.method.as_str() {
		"OPTIONS" => {
			Response::empty(Status::Ok)
		},
        "GET" => {
            println!("Handling a GET request for path: {}", request.path);
          	routing_get(&request).await
        }
        "POST" => {
            println!("Handling a POST request for path: {}", request.path);
            routing_post(&request).await
        }
        _ => {
            println!("Unknown or unsupported method: {}", request.method);
			Response::empty(Status::BadRequest)
        }
    }
}

async fn routing_get(request: &Request) -> Response {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return Response::empty(Status::NotFound);
        }
    };
    let response = match route {
        "login" => {
			controllers::user::log_in_get(request).await
		},
		"gallery" => {
            controllers::gallery::gallery(request).await
		},
		"create/history" => {
			controllers::create::create_get(request).await
		},
        _ => Response::empty(Status::NotFound),
    };
    response
}

async fn routing_post(request: &Request) -> Response {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return Response::empty(Status::NotFound);
        }
    };

    let response = match route {
        "login" => {
			controllers::user::log_in_post(request).await
		},
        "register" => {
			controllers::user::register(request).await
		},
		"gallery/like" => {
			controllers::gallery::like(request).await
		},
		"gallery/comment" => {
			controllers::gallery::comment(request).await
		},
		"create/post" => {
			controllers::create::create_post(request).await
		},
        _ =>  {
			Response::empty(Status::NotFound)
		},
    };
    response
}
