use crate::dto::gallery_dto::{CommentDTO, GalleryDTO, PaginatedGalleryDTO};
use crate::headers::{Request, Response, Status};
use crate::unwrap_or_return;
use crate::utils::{extract_json, log_error, send_email, AppState, EmailConfig};
use sqlx::{self};
use std::sync::Arc;

pub async fn gallery(request: &Request, state: &Arc<AppState>) -> Response {
    let query = unwrap_or_return!(request.query.as_ref(), Status::BadRequest);
    let (page, per_page) = unwrap_or_return!(validate_gallery_query(query), Status::BadRequest);

    println!("Page: {}, Per Page: {}", page, per_page);

    let safe_per_page = if per_page > 50 { 50 } else { per_page } as i64;
    let current_page = if page == 0 { 1 } else { page } as i64;
    let offset = (current_page - 1) * safe_per_page;

    let q_count = "SELECT COUNT(*) FROM posts";
    let q_posts = "
        SELECT 
            COALESCE(u.username, '[Deleted User]') AS author,
            p.id AS post_id, 
            p.image_path AS img_name,
            (SELECT COUNT(*) FROM post_likes WHERE post_id = p.id) AS likes
        FROM posts p
        LEFT JOIN users u ON p.user_id = u.id
        ORDER BY p.post_date DESC
        LIMIT $1 OFFSET $2
    ";

    let result = tokio::try_join!(
        sqlx::query_scalar::<_, i64>(q_count).fetch_one(&state.db),
        sqlx::query_as::<_, GalleryDTO>(q_posts)
            .bind(safe_per_page)
            .bind(offset)
            .fetch_all(&state.db)
    );

    match result {
        Ok((total_posts, posts)) => {
            let response_data = PaginatedGalleryDTO {
                posts,
                total_posts: total_posts as usize,
            };

            match serde_json::to_string(&response_data) {
                Ok(json) => Response::json(json),
                Err(e) => {
                    log_error("Error in PaginatedGallery serialization", e);
                    Response::empty(Status::InternalServerError)
                }
            }
        }
        Err(e) => {
            log_error("Database error fetching paginated posts", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn like(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let query = unwrap_or_return!(request.query.as_ref(), Status::BadRequest);
    let post_id = unwrap_or_return!(validate_like_query(query), Status::BadRequest);

    let mut tx = match state.db.begin().await {
        Ok(transaction) => transaction,
        Err(err) => {
            log_error("Failed to start transaction", err);
            return Response::empty(Status::InternalServerError);
        }
    };

    let q = "DELETE FROM post_likes WHERE user_id = $1 AND post_id = $2";
    let result = sqlx::query(q)
        .bind(&user_id)
        .bind(&post_id)
        .execute(&mut *tx)
        .await;
    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                let _ = tx.commit().await;
                return Response::empty(Status::Ok);
            }
            let q = "INSERT INTO post_likes (user_id, post_id) VALUES ($1, $2)";
            let result = sqlx::query(q)
                .bind(&user_id)
                .bind(&post_id)
                .execute(&mut *tx)
                .await;
            match result {
                Ok(_) => {
                    let _ = tx.commit().await;
                    return Response::empty(Status::Created);
                }
                Err(err) => {
                    let _ = tx.rollback().await;
                    log_error("Database error liking post", err);
                    return Response::empty(Status::InternalServerError);
                }
            }
        }
        Err(err) => {
            let _ = tx.rollback().await;
            log_error("Database error deleting like", err);
            return Response::empty(Status::InternalServerError);
        }
    }
}

#[derive(sqlx::FromRow)]
struct NotificationData {
    author_email: String,
    commenter_username: String,
}

pub async fn comment(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let query = unwrap_or_return!(request.query.as_ref(), Status::BadRequest);
    let post_id = unwrap_or_return!(validate_like_query(query), Status::BadRequest);

    let comment = match extract_json::<CommentDTO>(request) {
        Ok(res) => res,
        Err(status) => return Response::empty(status),
    };

    let q = "INSERT INTO comments (user_id, post_id, comment) VALUES ($1, $2, $3)";
    let result = sqlx::query(q)
        .bind(&user_id)
        .bind(&post_id)
        .bind(&comment.comment)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            let q = "SELECT 
                    u_author.email as author_email, 
                    u_commenter.username as commenter_username 
                FROM posts p 
                JOIN users u_author ON u_author.id = p.user_id
                JOIN users u_commenter ON u_commenter.id = $1
                WHERE p.id = $2 
                  AND u_author.notify_comment = TRUE 
                  AND u_author.is_deleted = FALSE";
                  
            let result = sqlx::query_as::<_, NotificationData>(q)
                .bind(&user_id)
                .bind(&post_id)
                .fetch_optional(&state.db)
                .await;
                
            if let Ok(Some(data)) = result {
                let email_conf = state.email_conf.clone();
                let NotificationData {
                    author_email,
                    commenter_username,
                } = data;

                tokio::spawn(async move {
                    prepare_email(
                        email_conf,
                        author_email,
                        commenter_username,
                        comment.comment,
                    ).await;
                });
            }

            Response::empty(Status::Ok)
        }
        Err(err) => {
            log_error("Database error saving comment post", err);
            Response::empty(Status::InternalServerError)
        }
    }
}

async fn prepare_email(
    email_conf: EmailConfig,
    recv_email: String,
    username: String,
    comment: String,
) {
    let from = format!("Camagru Admin <{}>", email_conf.get_email());
    let to = format!("<{}>", recv_email);
    let subject = "Your post was commented".to_string();
    let body = format!(
        "Hey, {} just left a comment to your post:\n {}",
        username, comment
    );
    send_email(email_conf, from, to, subject, body).await
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

fn validate_like_query(query: &str) -> Option<i32> {
    let mut key_value = query.splitn(2, '=');
    let key = key_value.next().unwrap_or("");
    let value = key_value.next().unwrap_or("");

    match key {
        "post_id" => Some(value.parse::<i32>().ok()?),
        _ => None,
    }
}

// async fn send_email(user_email: String, username: String, comment: String) {
//     let message = format!(
//         "Hey, {} just left a comment to your post:\n {}",
//         username, comment
//     );
//     let (username, password) = parse_env();

//     let email = Message::builder()
//         .from(format!("Camagru Admin <{}>", username).parse().unwrap())
//         .to(format!("<{}>", user_email).parse().unwrap())
//         .subject("Someone just commented your post")
//         .header(ContentType::TEXT_PLAIN)
//         .body(message)
//         .unwrap();

//     println!("{}, {}", username, password);
//     let creds = Credentials::new(username, password);

//     let mailer: AsyncSmtpTransport<Tokio1Executor> =
//         AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
//             .unwrap()
//             .credentials(creds)
//             .build();

//     match mailer.send(email).await {
//         Ok(_) => println!("Email sent successfully!"),
//         Err(e) => eprintln!("Could not send email: {:?}", e),
//     }
// }

// fn parse_env() -> (String, String) {
//     let username = match env::var("EMAIL_HOST") {
//         Ok(str) => str,
//         Err(_) => "default@gmail.com".to_string(),
//     };
//     let password = match env::var("PASSWORD_HOST") {
//         Ok(str) => str,
//         Err(_) => "123345".to_string(),
//     };
//     (username, password)
// }

// async fn prepare_email(email_conf: EmailConfig, recv_email: String, token: String) {
//     let verify_link = format!("http://localhost:80/api/verify?token={}", token);
//     let from = format!("Camagru Admin <{}>", email_conf.get_email());
//     let to = format!("<{}>", recv_email);
//     let subject = "Welcome to Camagru! Verify your account".to_string();
//     let body = format!(
//         "Please click the following link to verify your account: {}",
//         verify_link
//     );
//     send_email(email_conf, from, to, subject, body).await
// }

// fn validate_query(params: Vec<String>) -> Option<Vec<usize>> {
// 	let mut key_value = param.splitn(2, '=');
//     let key = key_value.next().unwrap_or("");
//     let value = key_value.next().unwrap_or("");

//     match key {
//         "id" => page = value.parse::<usize>().ok(),
//         _ => return None,
//     }
// }
