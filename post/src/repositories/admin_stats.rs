#[derive(Clone, Debug)]
pub struct AdminStatsSummary {
    pub user_count: i64,
    pub today_user_count: i64,
    pub post_count: i64,
    pub today_post_count: i64,
    pub comment_count: i64,
    pub today_comment_count: i64,
    pub like_count: i64,
}

#[derive(Clone, Debug)]
pub struct AdminHotPostRow {
    pub title: String,
    pub hot_score: i64,
}

#[derive(Clone, Debug)]
pub struct AdminHotTagRow {
    pub name: String,
    pub use_count: i64,
}

pub struct PostgresAdminStatsRepository;

impl PostgresAdminStatsRepository {
    pub async fn load_summary(pool: &sqlx::PgPool) -> sqlx::Result<AdminStatsSummary> {
        sqlx::query_as!(
            AdminStatsSummary,
            r#"
select
    (select count(*) from users)::bigint as "user_count!",
    (select count(*) from users where created_at >= date_trunc('day', now()))::bigint as "today_user_count!",
    (select count(*) from posts)::bigint as "post_count!",
    (select count(*) from posts where created_at >= date_trunc('day', now()))::bigint as "today_post_count!",
    (select count(*) from comments)::bigint as "comment_count!",
    (select count(*) from comments where created_at >= date_trunc('day', now()))::bigint as "today_comment_count!",
    (
        (select coalesce(sum(like_count), 0)::bigint from posts)
        + (select coalesce(sum(like_count), 0)::bigint from comments)
    )::bigint as "like_count!"
"#
        )
        .fetch_one(pool)
        .await
    }

    pub async fn top_hot_post(pool: &sqlx::PgPool) -> sqlx::Result<Option<AdminHotPostRow>> {
        sqlx::query_as!(
            AdminHotPostRow,
            r#"
select
    title,
    (view_count + comment_count * 20 + like_count * 10 + favorite_count * 5)::bigint as "hot_score!"
from posts
where status = 'published'
order by
    (view_count + comment_count * 20 + like_count * 10 + favorite_count * 5) desc,
    coalesce(published_at, created_at) desc,
    post_id desc
limit 1
"#
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn top_hot_tag(pool: &sqlx::PgPool) -> sqlx::Result<Option<AdminHotTagRow>> {
        sqlx::query_as!(
            AdminHotTagRow,
            r#"
select
    t.name,
    count(pt.post_id)::bigint as "use_count!"
from tags t
join post_tags pt on pt.tag_id = t.tag_id
join posts p on p.post_id = pt.post_id and p.status = 'published'
group by t.tag_id, t.name
order by count(pt.post_id) desc, t.name asc
limit 1
"#
        )
        .fetch_optional(pool)
        .await
    }
}
