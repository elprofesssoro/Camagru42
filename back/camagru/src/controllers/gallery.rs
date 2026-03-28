use crate::dto::gallery_dto::{GalleryDTO, PaginatedGalleryDTO};
use crate::headers::{Request, Response, Status};

pub async fn gallery(request: &Request) -> Response {
    let query = match request.query.as_ref() {
        Some(query) => query,
        None => return Response::empty(Status::BadRequest),
    };

    let (page, per_page) = match validate_query_input(query) {
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
            image_path: String::from("/pub/test.jpg"),
        };
        posts.push(post);
    }

    let response_data = PaginatedGalleryDTO { posts, total_posts };

    let json = serde_json::to_string(&response_data);
    let respone = match json {
        Ok(json) => Response::json(json),
        Err(e) => Response::empty(Status::InternalServerError),
    };

    respone
}

fn validate_query_input(query: &str) -> Option<(usize, usize)> {
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
