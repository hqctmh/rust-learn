use crate::http::Result;
use crate::http::user::UserAuth;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;
use validator::Validate;

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

    Ok(Json(post))
}
