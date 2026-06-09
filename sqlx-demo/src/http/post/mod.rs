mod comment;

use crate::http::Result;
use crate::http::user::UserAuth;
use axum::{Json, Router};
use axum::extract::State;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;
use validator::Validate;

pub fn router()->Router<PgPool>{
    Router::new()
        .route("/v1/post",get(get_posts).post(create_post))
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct CreatePostRequest {
    auth: UserAuth,
    #[validate(length(min = 1, max = 1000))]
    content: String,
}

#[serde_with::serde_as]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Post {
    post_id: Uuid,
    username: String,
    content: String,
    #[serde_as(as = "Rfc3339")]
    created_at: OffsetDateTime,
}

async fn create_post(db: State<PgPool>, Json(req): Json<CreatePostRequest>) -> Result<Json<Post>> {
    req.validate()?;
    let user_id = req.auth.verify(&*db).await?;

    let post=sqlx::query_as!(
        Post,
        r#"
            with inserted_post as (
                insert into post(user_id,content)
                       values ($1,$2)
                       returning post_id,user_id,content,created_at
            )
            select post_id,username,content,created_at
            from inserted_post
            inner join "user" using(user_id)
        "#,
        user_id,
        req.content
    ).fetch_one(&*db)
    .await?;

    Ok(Json(post))
}

async fn get_posts(db:State<PgPool>) -> Result<Json<Vec<Post>>>{
    let posts=sqlx::query_as!(
        Post,
        r#"
            select post_id,username,content,created_at
            from post
            inner join "user" using(user_id)
            order by created_at desc
        "#
    ).fetch_all(&*db).await?;

    Ok(Json(posts))
}