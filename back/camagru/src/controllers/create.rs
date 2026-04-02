use crate::headers::{Request, Response, Status};
use crate::dto::create_dto::{CreatePostDTO, HistoryDTO};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{from_slice, to_string};
use image;

pub async fn create_post(request: &Request) -> Response {
	let content_type = request.content_type.as_deref().unwrap_or("");

    if !content_type.starts_with("application/json") {
        return Response::empty(Status::UnsupportedMediaType);
    }

	let body = match request.body.as_ref() {
		Some(body) => body,
		None => return Response::empty(Status::BadRequest)
	};

	let payload = match from_slice::<CreatePostDTO>(body) {
		Ok(res) => res,
		Err(_) => return Response::empty(Status::BadRequest)
	};
	let image_str = payload.image.split_once(',').map(|(_, data)| data.to_string()).unwrap_or(payload.image);
    let sticker_name = payload.sticker_name;
    let (pos_x, pos_y, width, height) = (payload.pos_x, payload.pos_y, payload.width, payload.height);

    let process_result = tokio::task::spawn_blocking(move || {
        let image_bytes = match STANDARD.decode(&image_str) {
            Ok(b) => b,
            Err(_) => return Err(Status::BadRequest),
        };

        let mut img = match image::load_from_memory(&image_bytes) {
            Ok(i) => i,
            Err(_) => return Err(Status::Unauthorized),
        };

        let sticker_path = format!("../../pub/stickers/{}", sticker_name);
        let sticker = match image::open(&sticker_path) {
            Ok(s) => s,
            Err(_) => return Err(Status::NotFound),
        };

        let sticker = image::imageops::resize(&sticker, width, height, image::imageops::FilterType::Nearest);
        image::imageops::overlay(&mut img, &sticker, pos_x as i64, pos_y as i64);

        let final_path = "../../pub/posts/my_new_photo.png"; 
        if img.save(final_path).is_err() {
            return Err(Status::InternalServerError);
        }

        Ok(final_path.to_string()) 
        
    }).await;

    match process_result {
        Ok(Ok(saved_path)) => {
            println!("Image successfully processed in background and saved to {}", saved_path);
            return Response::empty(Status::Ok)
        }
        Ok(Err(status)) => return Response::empty(status), 
        Err(_) => return Response::empty(Status::InternalServerError),
    }
}


pub async fn create_get(request: &Request) -> Response {
	let mut history_posts = Vec::<HistoryDTO>::new();
	
	for i in 0..10 {
		let post = HistoryDTO {
			img_name: format!("my_new_photo.png"),
			likes: i,
			post_id: i
		};
		history_posts.push(post)
	};
	match to_string(&history_posts){
		Ok(json) => {
			return Response::json(json)
		},
		Err(e) => {
			println!("Error parsing to string history post: {:?}", e);
			return Response::empty(Status::InternalServerError)
		}
	};
}

pub async fn create_delete(request: &Request) -> Response {
	let query = match request.query.as_ref() {
		Some(query) => query,
		None => return Response::empty(Status::BadRequest)
	};
	let post_id = match validate_delete_query(&query) {
		Some(post_id) => post_id,
		None => return Response::empty(Status::BadRequest)
	};

	println!("Deleting post {:?}", post_id);
	Response::empty(Status::Ok)
}


pub fn validate_delete_query(query: &str) -> Option<i32> {
    let mut key_value = query.splitn(2, '=');
    let key = key_value.next().unwrap_or("");
    let value = key_value.next().unwrap_or("");

    match key {
        "post_id" => value.parse::<i32>().ok(),
        _ => return None,
    }
}