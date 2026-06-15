use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::home::{HomeTopic, TopicMarker, dense_workbench_topics},
    services::auth::AuthService,
};

pub struct PostgresDemoSeedRepository;

const HOME_DEMO_SEED_MARKER_TITLE: &str = "Leptos 0.6 发布：更快的编译、更小的体积、Signal 优化";

impl PostgresDemoSeedRepository {
    pub async fn ensure_homepage_seed(pool: &sqlx::PgPool) -> sqlx::Result<()> {
        let topics = dense_workbench_topics();
        debug_assert!(
            topics
                .iter()
                .any(|topic| topic.title == HOME_DEMO_SEED_MARKER_TITLE)
        );
        let password_hash = AuthService::hash_password("post-demo-password");
        let now = OffsetDateTime::from_unix_timestamp(1_716_192_000)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());

        for (index, topic) in topics.iter().enumerate() {
            let author_id = demo_uuid(0x7100, index);
            let category_id = demo_uuid(0x7200, index);
            let post_id = demo_uuid(0x7300, index);
            let published_at = now - Duration::minutes((index as i64) * 30 + 1);
            let username = format!("demo_home_author_{index}");

            let user_row = sqlx::query!(
                r#"
insert into users (
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    bio,
    status,
    is_admin
)
values ($1, $2, $3, $4, null, $5, 'active', $6)
on conflict (username) do update set
    nickname = excluded.nickname,
    bio = excluded.bio,
    status = 'active',
    updated_at = now()
returning user_id
"#,
                author_id,
                username,
                password_hash,
                topic.last_reply.author,
                "Post Forum 首页设计稿演示作者",
                topic.last_reply.author == "管理员"
            )
            .fetch_one(pool)
            .await?;

            let category_row = sqlx::query!(
                r#"
insert into categories (
    category_id,
    name,
    slug,
    sort_order,
    color,
    enabled
)
values ($1, $2, $3, $4, $5, true)
on conflict (name) do update set
    sort_order = excluded.sort_order,
    color = excluded.color,
    enabled = true
returning category_id
"#,
                category_id,
                topic.category.name,
                demo_slug("category", &topic.category.name),
                index as i32,
                topic.category.color
            )
            .fetch_one(pool)
            .await?;

            sqlx::query!(
                r#"
insert into posts (
    post_id,
    author_id,
    category_id,
    title,
    summary,
    status,
    is_recommended,
    is_pinned,
    is_locked,
    view_count,
    comment_count,
    like_count,
    favorite_count,
    published_at
)
values ($1, $2, $3, $4, $5, 'published', true, $6, $7, $8, $9, 0, 0, $10)
on conflict (post_id) do update set
    author_id = excluded.author_id,
    category_id = excluded.category_id,
    title = excluded.title,
    summary = excluded.summary,
    status = 'published',
    is_recommended = true,
    is_pinned = excluded.is_pinned,
    is_locked = excluded.is_locked,
    view_count = excluded.view_count,
    comment_count = excluded.comment_count,
    published_at = excluded.published_at,
    updated_at = now()
"#,
                post_id,
                user_row.user_id,
                category_row.category_id,
                topic.title,
                topic.summary,
                topic.marker == TopicMarker::Pinned,
                topic.marker == TopicMarker::Locked,
                view_count_from_label(&topic.view_count_label),
                topic.reply_count as i64,
                published_at
            )
            .execute(pool)
            .await?;

            sqlx::query!(
                r#"
insert into post_contents (
    post_id,
    markdown,
    sanitized_html
)
values ($1, $2, $3)
on conflict (post_id) do update set
    markdown = excluded.markdown,
    sanitized_html = excluded.sanitized_html,
    updated_at = now()
"#,
                post_id,
                demo_markdown(topic),
                demo_html(topic)
            )
            .execute(pool)
            .await?;

            seed_topic_tags(pool, post_id, topic, index).await?;
        }

        Ok(())
    }
}

async fn seed_topic_tags(
    pool: &sqlx::PgPool,
    post_id: Uuid,
    topic: &HomeTopic,
    topic_index: usize,
) -> sqlx::Result<()> {
    for (tag_index, tag) in topic.tags.iter().enumerate() {
        let tag_id = demo_uuid(0x7400_0000 + (topic_index as u128) * 100, tag_index);
        let row = sqlx::query!(
            r#"
insert into tags (
    tag_id,
    name,
    slug,
    sort_order,
    enabled,
    use_count
)
values ($1, $2, $3, $4, true, $5)
on conflict (name) do update set
    sort_order = excluded.sort_order,
    enabled = true,
    use_count = greatest(tags.use_count, excluded.use_count)
returning tag_id
"#,
            tag_id,
            tag.name,
            demo_slug("tag", &tag.name),
            tag_index as i32,
            tag.count as i64
        )
        .fetch_one(pool)
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
            post_id,
            row.tag_id
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn demo_uuid(prefix: u128, index: usize) -> Uuid {
    Uuid::from_u128(0x9000_0000_0000_0000_0000_0000_0000_0000 + prefix + index as u128)
}

fn demo_slug(prefix: &str, name: &str) -> String {
    let mut parts = Vec::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            parts.push(ch.to_ascii_lowercase().to_string());
        } else if !ch.is_ascii_whitespace() {
            parts.push(format!("{:x}", ch as u32));
        }
    }
    let normalized = parts.join("-");
    if normalized.is_empty() {
        format!("{prefix}-demo")
    } else {
        format!("{prefix}-{normalized}")
    }
}

fn demo_markdown(topic: &HomeTopic) -> String {
    format!("# {}\n\n{}\n", topic.title, topic.summary)
}

fn demo_html(topic: &HomeTopic) -> String {
    format!("<h1>{}</h1><p>{}</p>", topic.title, topic.summary)
}

fn view_count_from_label(label: &str) -> i64 {
    if let Some(value) = label.strip_suffix('k') {
        return value
            .parse::<f64>()
            .map(|value| (value * 1000.0).round() as i64)
            .unwrap_or_default();
    }
    label.parse::<i64>().unwrap_or_default()
}
