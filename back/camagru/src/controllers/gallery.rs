use crate::headers::{Request, Response, Status};
use crate::dto::gallery_dto::GalleryDTO;
use serde_json::from_slice;
use std::fs;

pub async fn gallery(request: &Request) -> Response {
	let query = match request.query.as_ref() {
		Some(query) => query,
		None => return Response::empty(Status::BadRequest)
	};

	let (page, per_page) =match validate_query_input(query) {
		Some((page, per_page)) => (page, per_page),
		None => return Response::empty(Status::BadRequest)
	};
	println!("Page: {}, Per Page: {}", page, per_page);
	let mut posts: Vector<GalleryDTO>;

	let mut i: i32 = 0;
	loop {
		if (i >= per_page) { break };
		let mut post: GalleryDTO;
		post.author = "User " + i.to_string();
		post.likes = i;
		post.image_path = String::from("/pub/test.jpg");
		posts.push(post);
		i +=1;
	}
    Response::empty(Status::Ok)
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
