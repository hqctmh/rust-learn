use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    moderation::{
        ModerationCommentAction, ModerationCommentRow, ModerationPostAction, ModerationPostRow,
    },
    posts::PostStatus,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModerationPostDbRow {
    pub post_id: Uuid,
    pub title: String,
    pub author_name: String,
    pub category_name: Option<String>,
    pub status: String,
    pub pinned: bool,
    pub comment_count: i64,
    pub view_count: i64,
    pub updated_at: Option<OffsetDateTime>,
}

impl From<ModerationPostDbRow> for ModerationPostRow {
    fn from(row: ModerationPostDbRow) -> Self {
        Self {
            post_id: row.post_id,
            title: row.title,
            author_name: row.author_name,
            category_name: row.category_name,
            status: post_status_from_str(&row.status),
            pinned: row.pinned,
            comment_count: row.comment_count,
            view_count: row.view_count,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModerationPostActionRow {
    pub post_id: Uuid,
    pub status: String,
    pub pinned: bool,
}

impl From<ModerationPostActionRow> for ModerationPostAction {
    fn from(row: ModerationPostActionRow) -> Self {
        Self {
            post_id: row.post_id,
            status: post_status_from_str(&row.status),
            pinned: row.pinned,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModerationCommentDbRow {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub post_title: String,
    pub author_name: String,
    pub content: String,
    pub deleted: bool,
    pub created_at: OffsetDateTime,
}

impl From<ModerationCommentDbRow> for ModerationCommentRow {
    fn from(row: ModerationCommentDbRow) -> Self {
        Self {
            comment_id: row.comment_id,
            post_id: row.post_id,
            post_title: row.post_title,
            author_name: row.author_name,
            content: row.content,
            deleted: row.deleted,
            created_at: row.created_at,
        }
    }
}

pub struct PostgresModerationRepository;

impl PostgresModerationRepository {
    pub async fn list_posts(pool: &sqlx::PgPool) -> sqlx::Result<Vec<ModerationPostRow>> {
        let rows = sqlx::query_as!(
            ModerationPostDbRow,
            r#"
select
    p.post_id,
    p.title,
    u.nickname as author_name,
    c.name as "category_name?",
    p.status,
    p.is_pinned as "pinned!",
    p.comment_count,
    p.view_count,
    p.updated_at as "updated_at?"
from posts p
join users u on u.user_id = p.author_id
left join categories c on c.category_id = p.category_id
order by p.updated_at desc, p.created_at desc
"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(ModerationPostRow::from).collect())
    }

    pub async fn find_post_action(
        pool: &sqlx::PgPool,
        post_id: Uuid,
    ) -> sqlx::Result<Option<ModerationPostAction>> {
        let row = sqlx::query_as!(
            ModerationPostActionRow,
            r#"
select
    post_id,
    status,
    is_pinned as "pinned!"
from posts
where post_id = $1
limit 1
"#,
            post_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ModerationPostAction::from))
    }

    pub async fn set_post_status(
        pool: &sqlx::PgPool,
        post_id: Uuid,
        status: &PostStatus,
    ) -> sqlx::Result<Option<ModerationPostAction>> {
        let status = post_status_as_str(status);
        let row = sqlx::query_as!(
            ModerationPostActionRow,
            r#"
update posts
set
    status = $2,
    is_pinned = case when $2 = 'deleted' then false else is_pinned end,
    updated_at = now()
where post_id = $1
returning
    post_id,
    status,
    is_pinned as "pinned!"
"#,
            post_id,
            status
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ModerationPostAction::from))
    }

    pub async fn set_post_pin(
        pool: &sqlx::PgPool,
        post_id: Uuid,
        pinned: bool,
    ) -> sqlx::Result<Option<ModerationPostAction>> {
        let row = sqlx::query_as!(
            ModerationPostActionRow,
            r#"
update posts
set
    is_pinned = $2,
    updated_at = now()
where post_id = $1
  and status <> 'deleted'
returning
    post_id,
    status,
    is_pinned as "pinned!"
"#,
            post_id,
            pinned
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ModerationPostAction::from))
    }

    pub async fn list_comments(pool: &sqlx::PgPool) -> sqlx::Result<Vec<ModerationCommentRow>> {
        let rows = sqlx::query_as!(
            ModerationCommentDbRow,
            r#"
select
    c.comment_id,
    c.post_id,
    p.title as post_title,
    u.nickname as author_name,
    c.content,
    (c.status = 'deleted') as "deleted!",
    c.created_at
from comments c
join posts p on p.post_id = c.post_id
join users u on u.user_id = c.author_id
order by c.created_at desc
"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(ModerationCommentRow::from).collect())
    }

    pub async fn set_comment_deleted(
        pool: &sqlx::PgPool,
        comment_id: Uuid,
        deleted: bool,
    ) -> sqlx::Result<Option<ModerationCommentAction>> {
        let mut tx = pool.begin().await?;
        let current = sqlx::query!(
            r#"
select
    comment_id,
    post_id,
    status
from comments
where comment_id = $1
for update
"#,
            comment_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(current) = current else {
            tx.commit().await?;
            return Ok(None);
        };

        let current_deleted = current.status == "deleted";
        if current_deleted != deleted {
            let next_status = if deleted { "deleted" } else { "visible" };
            sqlx::query!(
                r#"
update comments
set status = $2,
    updated_at = now()
where comment_id = $1
"#,
                current.comment_id,
                next_status
            )
            .execute(&mut *tx)
            .await?;

            let delta = if deleted { -1_i64 } else { 1_i64 };
            sqlx::query!(
                r#"
update posts
set comment_count = greatest(comment_count + $2, 0::bigint),
    updated_at = now()
where post_id = $1
"#,
                current.post_id,
                delta
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(Some(ModerationCommentAction {
            comment_id: current.comment_id,
            post_id: current.post_id,
            deleted,
        }))
    }
}

fn post_status_as_str(status: &PostStatus) -> &'static str {
    match status {
        PostStatus::Published => "published",
        PostStatus::Offline => "offline",
        PostStatus::Deleted => "deleted",
        PostStatus::Draft => "draft",
    }
}

fn post_status_from_str(status: &str) -> PostStatus {
    match status {
        "published" => PostStatus::Published,
        "offline" => PostStatus::Offline,
        "deleted" => PostStatus::Deleted,
        _ => PostStatus::Draft,
    }
}
