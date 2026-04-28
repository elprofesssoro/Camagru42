use crate::dto::create_dto::{CommentDTO, HistoryDTO};
use sqlx::{postgres::PgRow, Error, PgPool, Row};

pub struct CreateRepo;

impl CreateRepo {
    pub async fn create_post(db: &PgPool, user_id: i32, image_path: &str) -> Result<(), Error> {
        let q = "INSERT INTO posts (user_id, image_path) VALUES ($1, $2)";
        sqlx::query(q)
            .bind(user_id)
            .bind(image_path)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn get_user_posts(db: &PgPool, user_id: i32) -> Result<Vec<HistoryDTO>, Error> {
        let q = "SELECT image_path, post_date, id FROM posts WHERE user_id = $1";
        sqlx::query_as::<_, HistoryDTO>(q)
            .bind(user_id)
            .fetch_all(db)
            .await
    }

    pub async fn delete_post(db: &PgPool, post_id: i32, user_id: i32) -> Result<Option<String>, Error> {
        let q = "DELETE FROM posts WHERE id = $1 AND user_id = $2 RETURNING image_path";
        let row = sqlx::query(q)
            .bind(post_id)
            .bind(user_id)
            .fetch_optional(db)
            .await?;

        Ok(row.map(|r| r.get("image_path")))
    }

    pub async fn get_post_details(
        db: &PgPool,
        post_id: i32,
    ) -> Result<(PgRow, Vec<CommentDTO>), Error> {
        let q_post = "
            SELECT 
                post_date,
                (SELECT COUNT(*) FROM post_likes WHERE post_id = $1) AS likes 
            FROM posts WHERE id = $1;";

        let q_comments = "
            SELECT 
                COALESCE(u.username, '[Deleted User]') AS username,
                c.comment 
            FROM comments c
            LEFT JOIN users u ON c.user_id = u.id
            WHERE c.post_id = $1
            ORDER BY c.id DESC;";

        tokio::try_join!(
            sqlx::query(q_post).bind(&post_id).fetch_one(db),
            sqlx::query_as::<_, CommentDTO>(q_comments).bind(post_id).fetch_all(db)
        )
    }
}
