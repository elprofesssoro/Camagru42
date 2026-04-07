use crate::dto::gallery_dto::{CommentDTO, GalleryDTO, PaginatedGalleryDTO};
use crate::headers::{log_error, Request, Response, Status};

pub async fn gallery(request: &Request) -> Response {
    let query = match request.query.as_ref() {
        Some(query) => query,
        None => return Response::empty(Status::BadRequest),
    };

    let (page, per_page) = match validate_gallery_query(query) {
        Some((page, per_page)) => (page, per_page),
        None => return Response::empty(Status::BadRequest),
    };
    println!("Page: {}, Per Page: {}", page, per_page);
    let mut posts: Vec<GalleryDTO> = Vec::new();

    let total_posts: usize = 200;
    let safe_per_page = if per_page > 50 { 50 } else { per_page };
    let current_page = if page == 0 { 1 } else { page };

    let start_index = (current_page - 1) * safe_per_page;
    let end_index = start_index + safe_per_page;
    for i in start_index..end_index.min(total_posts) {
        let post = GalleryDTO {
            author: format!("User {}", i),
            likes: i,
            img_name: String::from("my_new_photo.png"),
            post_id: i + 100,
        };
        posts.push(post);
    }

    let response_data = PaginatedGalleryDTO { posts, total_posts };

    let json = serde_json::to_string(&response_data);
    let respone = match json {
        Ok(json) => Response::json(json),
        Err(e) => {
            log_error("Error in PaginatedGallery serialization", e);
            Response::empty(Status::InternalServerError)
        }
    };

    respone
}

pub async fn like(request: &Request) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };
    let query = match request.query.as_ref() {
        Some(query) => query,
        None => return Response::empty(Status::BadRequest),
    };

    let (user_id, post_id) = match validate_like_query(query) {
        Some((user_id, post_id)) => (user_id, post_id),
        None => return Response::empty(Status::BadRequest),
    };

    println!("User_id: {:?}, Post_id: {:?}", user_id, post_id);
    Response::empty(Status::Ok)
}

pub async fn comment(request: &Request) -> Response {
    let query = match request.query.as_ref() {
        Some(query) => query,
        None => return Response::empty(Status::BadRequest),
    };

    let (_user_id, _post_id) = match validate_like_query(query) {
        Some((user_id, post_id)) => (user_id, post_id),
        None => return Response::empty(Status::BadRequest),
    };
    let body = match request.body.as_ref() {
        Some(body) => body,
        None => return Response::empty(Status::BadRequest),
    };
    let comment = match serde_json::from_slice::<CommentDTO>(body) {
        Ok(res) => res,
        Err(_) => return Response::empty(Status::BadRequest),
    };
    println!("Comment: {:?}", comment);
    Response::empty(Status::Ok)
}

fn validate_gallery_query(query: &str) -> Option<(usize, usize)> {
    let params: Vec<&str> = query.split('&').collect();
    let mut page = None;
    let mut per_page = None;

    for param in params {
        let mut key_value = param.splitn(2, '=');
        let key = key_value.next().unwrap_or("");
        let value = key_value.next().unwrap_or("");

        match key {
            "page" => page = value.parse::<usize>().ok(),
            "per_page" => per_page = value.parse::<usize>().ok(),
            _ => return None,
        }
    }

    if let (Some(page), Some(per_page)) = (page, per_page) {
        Some((page, per_page))
    } else {
        None
    }
}

fn validate_like_query(query: &str) -> Option<(usize, usize)> {
    let params: Vec<&str> = query.split('&').collect();
    let mut user_id = None;
    let mut post_id = None;

    for param in params {
        let mut key_value = param.splitn(2, '=');
        let key = key_value.next().unwrap_or("");
        let value = key_value.next().unwrap_or("");

        match key {
            "user_id" => user_id = value.parse::<usize>().ok(),
            "post_id" => post_id = value.parse::<usize>().ok(),
            _ => return None,
        }
    }

    if let (Some(user_id), Some(post_id)) = (user_id, post_id) {
        Some((user_id, post_id))
    } else {
        None
    }
}

// fn validate_query(params: Vec<String>) -> Option<Vec<usize>> {
// 	let mut key_value = param.splitn(2, '=');
//     let key = key_value.next().unwrap_or("");
//     let value = key_value.next().unwrap_or("");

//     match key {
//         "id" => page = value.parse::<usize>().ok(),
//         _ => return None,
//     }
// }
