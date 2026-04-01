use crate::headers::{Request, Response, Status};
use crate::dto::create_dto::{CreatePostDTO};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::from_slice;
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
    let sticker_path = payload.sticker_path;
    let (pos_x, pos_y, width, height) = (payload.pos_x, payload.pos_y, payload.width, payload.height);

    let process_result = tokio::task::spawn_blocking(move || {
        let image_bytes = match STANDARD.decode(&image_str) {
            Ok(b) => b,
            Err(_) => return Err(Status::BadRequest),
        };

        let mut img = match image::load_from_memory(&image_bytes) {
            Ok(i) => i,
            Err(_) => return Err(Status::BadRequest),
        };

        //let sticker_path = format!("pub/stickers/{}.png", sticker_path);
        let sticker = match image::open(&sticker_path) {
            Ok(s) => s,
            Err(_) => return Err(Status::BadRequest),
        };

        let sticker = image::imageops::resize(&sticker, width, height, image::imageops::FilterType::Nearest);
        image::imageops::overlay(&mut img, &sticker, pos_x as i64, pos_y as i64);

        let final_path = "pub/my_new_photo.png"; 
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
