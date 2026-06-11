use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::posts::{PostDetail, PostStatus, PostSummary};

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct PostSummaryRow {
    pub post_id: Uuid,
    pub title: String,
    pub summary: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_avatar_url: Option<String>,
    pub category_name: Option<String>,
    pub tags: Vec<String>,
    pub view_count: i64,
    pub comment_count: i64,
    pub like_count: i64,
    pub favorite_count: i64,
    pub published_at: Option<OffsetDateTime>,
}

impl From<PostSummaryRow> for PostSummary {
    fn from(row: PostSummaryRow) -> Self {
        Self {
            post_id: row.post_id,
            title: row.title,
            summary: row.summary,
            author_id: row.author_id,
            author_name: row.author_name,
            author_avatar_url: row.author_avatar_url,
            category_name: row.category_name,
            tags: row.tags,
            view_count: row.view_count,
            comment_count: row.comment_count,
            like_count: row.like_count,
            favorite_count: row.favorite_count,
            published_at: row.published_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct PostDetailRow {
    pub post_id: Uuid,
    pub title: String,
    pub summary: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_avatar_url: Option<String>,
    pub category_name: Option<String>,
    pub tags: Vec<String>,
    pub view_count: i64,
    pub comment_count: i64,
    pub like_count: i64,
    pub favorite_count: i64,
    pub published_at: Option<OffsetDateTime>,
    pub markdown: String,
    pub sanitized_html: String,
    pub status: String,
}

impl From<PostDetailRow> for PostDetail {
    fn from(row: PostDetailRow) -> Self {
        Self {
            summary: PostSummary {
                post_id: row.post_id,
                title: row.title,
                summary: row.summary,
                author_id: row.author_id,
                author_name: row.author_name,
                author_avatar_url: row.author_avatar_url,
                category_name: row.category_name,
                tags: row.tags,
                view_count: row.view_count,
                comment_count: row.comment_count,
                like_count: row.like_count,
                favorite_count: row.favorite_count,
                published_at: row.published_at,
            },
            markdown: row.markdown,
            sanitized_html: row.sanitized_html,
            status: post_status_from_str(&row.status),
            liked_by_me: false,
            favorited_by_me: false,
            following_author: false,
        }
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

pub struct PostgresPostRepository;

impl PostgresPostRepository {
    pub fn published_summaries_sql() -> &'static str {
        r#"
select
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname as author_name,
    u.avatar_url as author_avatar_url,
    c.name as category_name,
    coalesce(array_remove(array_agg(t.name order by t.name), null), array[]::text[]) as tags,
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at
from posts p
join users u on u.user_id = p.author_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
where p.status = 'published'
group by
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname,
    u.avatar_url,
    c.name,
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at,
    p.is_pinned,
    p.created_at
order by p.is_pinned desc, p.published_at desc nulls last, p.created_at desc
limit $1 offset $2
"#
    }

    pub fn post_detail_sql() -> &'static str {
        r#"
select
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname as author_name,
    u.avatar_url as author_avatar_url,
    c.name as category_name,
    coalesce(array_remove(array_agg(t.name order by t.name), null), array[]::text[]) as tags,
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at,
    pc.markdown,
    pc.sanitized_html,
    p.status
from posts p
join users u on u.user_id = p.author_id
join post_contents pc on pc.post_id = p.post_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
where p.post_id = $1
  and p.status <> 'deleted'
group by
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname,
    u.avatar_url,
    c.name,
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at,
    pc.markdown,
    pc.sanitized_html,
    p.status
limit 1
"#
    }

    pub async fn list_published_summaries(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<Vec<PostSummary>> {
        let rows = sqlx::query_as!(
            PostSummaryRow,
            r#"
select
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname as author_name,
    u.avatar_url as author_avatar_url,
    c.name as category_name,
    coalesce(array_remove(array_agg(t.name order by t.name), null), array[]::text[]) as "tags!: Vec<String>",
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at
from posts p
join users u on u.user_id = p.author_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
where p.status = 'published'
group by
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname,
    u.avatar_url,
    c.name,
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at,
    p.is_pinned,
    p.created_at
order by p.is_pinned desc, p.published_at desc nulls last, p.created_at desc
limit $1 offset $2
"#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(PostSummary::from).collect())
    }

    pub async fn find_detail(
        pool: &sqlx::PgPool,
        post_id: Uuid,
    ) -> sqlx::Result<Option<PostDetail>> {
        let row = sqlx::query_as!(
            PostDetailRow,
            r#"
select
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname as author_name,
    u.avatar_url as author_avatar_url,
    c.name as category_name,
    coalesce(array_remove(array_agg(t.name order by t.name), null), array[]::text[]) as "tags!: Vec<String>",
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at,
    pc.markdown,
    pc.sanitized_html,
    p.status
from posts p
join users u on u.user_id = p.author_id
join post_contents pc on pc.post_id = p.post_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
where p.post_id = $1
  and p.status <> 'deleted'
group by
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname,
    u.avatar_url,
    c.name,
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at,
    pc.markdown,
    pc.sanitized_html,
    p.status
limit 1
"#,
            post_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(PostDetail::from))
    }

    pub async fn insert_post(pool: &sqlx::PgPool, detail: &PostDetail) -> sqlx::Result<()> {
        let mut tx = pool.begin().await?;
        let category_id = if let Some(category_name) = &detail.summary.category_name {
            let category_id = Uuid::new_v4();
            let category_slug = stable_slug("category", category_name, category_id);
            let row = sqlx::query!(
                r#"
insert into categories (
    category_id,
    name,
    slug
)
values ($1, $2, $3)
on conflict (name) do update set name = excluded.name
returning category_id
"#,
                category_id,
                category_name,
                category_slug
            )
            .fetch_one(&mut *tx)
            .await?;
            Some(row.category_id)
        } else {
            None
        };

        let status = post_status_as_str(&detail.status);
        sqlx::query!(
            r#"
insert into posts (
    post_id,
    author_id,
    category_id,
    title,
    summary,
    status,
    view_count,
    comment_count,
    like_count,
    favorite_count,
    published_at
)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
"#,
            detail.summary.post_id,
            detail.summary.author_id,
            category_id,
            detail.summary.title,
            detail.summary.summary,
            status,
            detail.summary.view_count,
            detail.summary.comment_count,
            detail.summary.like_count,
            detail.summary.favorite_count,
            detail.summary.published_at
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
insert into post_contents (
    post_id,
    markdown,
    sanitized_html
)
values ($1, $2, $3)
"#,
            detail.summary.post_id,
            detail.markdown,
            detail.sanitized_html
        )
        .execute(&mut *tx)
        .await?;

        for tag_name in &detail.summary.tags {
            let tag_id = Uuid::new_v4();
            let tag_slug = stable_slug("tag", tag_name, tag_id);
            let row = sqlx::query!(
                r#"
insert into tags (
    tag_id,
    name,
    slug
)
values ($1, $2, $3)
on conflict (name) do update set name = excluded.name
returning tag_id
"#,
                tag_id,
                tag_name,
                tag_slug
            )
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query!(
                r#"
insert into post_tags (
    post_id,
    tag_id
)
values ($1, $2)
on conflict do nothing
"#,
                detail.summary.post_id,
                row.tag_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await
    }

    pub async fn update_post(pool: &sqlx::PgPool, detail: &PostDetail) -> sqlx::Result<()> {
        let mut tx = pool.begin().await?;
        let category_id = if let Some(category_name) = &detail.summary.category_name {
            let category_id = Uuid::new_v4();
            let category_slug = stable_slug("category", category_name, category_id);
            let row = sqlx::query!(
                r#"
insert into categories (
    category_id,
    name,
    slug
)
values ($1, $2, $3)
on conflict (name) do update set name = excluded.name
returning category_id
"#,
                category_id,
                category_name,
                category_slug
            )
            .fetch_one(&mut *tx)
            .await?;
            Some(row.category_id)
        } else {
            None
        };

        let status = post_status_as_str(&detail.status);
        sqlx::query!(
            r#"
update posts
set category_id = $2,
    title = $3,
    summary = $4,
    status = $5,
    published_at = $6,
    updated_at = now()
where post_id = $1
  and author_id = $7
  and status <> 'deleted'
"#,
            detail.summary.post_id,
            category_id,
            detail.summary.title,
            detail.summary.summary,
            status,
            detail.summary.published_at,
            detail.summary.author_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
update post_contents
set markdown = $2,
    sanitized_html = $3,
    updated_at = now()
where post_id = $1
"#,
            detail.summary.post_id,
            detail.markdown,
            detail.sanitized_html
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
delete from post_tags
where post_id = $1
"#,
            detail.summary.post_id
        )
        .execute(&mut *tx)
        .await?;

        for tag_name in &detail.summary.tags {
            let tag_id = Uuid::new_v4();
            let tag_slug = stable_slug("tag", tag_name, tag_id);
            let row = sqlx::query!(
                r#"
insert into tags (
    tag_id,
    name,
    slug
)
values ($1, $2, $3)
on conflict (name) do update set name = excluded.name
returning tag_id
"#,
                tag_id,
                tag_name,
                tag_slug
            )
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query!(
                r#"
insert into post_tags (
    post_id,
    tag_id
)
values ($1, $2)
on conflict do nothing
"#,
                detail.summary.post_id,
                row.tag_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await
    }

    pub async fn mark_deleted(
        pool: &sqlx::PgPool,
        post_id: Uuid,
        author_id: Uuid,
    ) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            r#"
update posts
set status = 'deleted',
    updated_at = now()
where post_id = $1
  and author_id = $2
  and status <> 'deleted'
"#,
            post_id,
            author_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
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

fn stable_slug(prefix: &str, name: &str, id: Uuid) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in name.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        format!("{prefix}-{id}")
    } else {
        slug
    }
}
