use crate::http::user::UserAuth;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;
use validator::Validate;

use crate::http::Result;

pub fn router() -> Router<PgPool> {
    Router::new().route(
        "/v1/post/{postId}/comment",
        get(get_posts_comments).post(create_post_comment),
    )
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct CreateCommentRequest {
    auth: UserAuth,
    #[validate(length(min = 1, max = 1000))]
    content: String,
}

#[serde_with::serde_as]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Comment {
    comment_id: Uuid,
    username: String,
    content: String,
    #[serde_as(as = "Rfc3339")]
    created_at: OffsetDateTime,
}

async fn create_post_comment(
    db: State<PgPool>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<Comment>> {
    req.validate()?;
    let user_id = req.auth.verify(&*db).await?;

    let comment = sqlx::query_as!(
        Comment,
        r#"
            with inserted_comment as (
                insert into comment(user_id,post_id,content)
                       values ($1,$2,$3)
                       returning comment_id,user_id,content,created_at
            )
            select comment_id,username,content,created_at
            from inserted_comment
            inner join "user" using(user_id)
        "#,
        user_id,
        post_id,
        req.content
    )
    .fetch_one(&*db)
    .await?;

    Ok(Json(comment))
}

async fn get_posts_comments(
    db: State<PgPool>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<Vec<Comment>>> {
    let comments = sqlx::query_as!(
        Comment,
        r#"
            select comment_id,username,content,created_at
            from comment
            inner join "user" using(user_id)
            where post_id = $1
            order by created_at
        "#,
        post_id
    )
    .fetch_all(&*db)
    .await?;
    Ok(Json(comments))
}
