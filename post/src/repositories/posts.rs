use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    home::{HomeQuery, HomeSort, HomeTab, HomeTimeRange},
    posts::{PostDetail, PostStatus, PostSummary},
};

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
    pub last_reply_author_name: Option<String>,
    pub last_reply_author_avatar_url: Option<String>,
    pub last_reply_at: Option<OffsetDateTime>,
    pub pinned: bool,
    pub locked: bool,
    pub read_by_me: bool,
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
            last_reply_author_name: row.last_reply_author_name,
            last_reply_author_avatar_url: row.last_reply_author_avatar_url,
            last_reply_at: row.last_reply_at,
            pinned: row.pinned,
            locked: row.locked,
            read_by_me: row.read_by_me,
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
    pub pinned: bool,
    pub locked: bool,
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
                last_reply_author_name: None,
                last_reply_author_avatar_url: None,
                last_reply_at: None,
                pinned: row.pinned,
                locked: row.locked,
                read_by_me: false,
            },
            markdown: row.markdown,
            sanitized_html: row.sanitized_html,
            status: post_status_from_str(&row.status),
            locked: row.locked,
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
    p.published_at,
    lr.author_name as "last_reply_author_name?",
    lr.author_avatar_url as "last_reply_author_avatar_url?",
    lr.replied_at as "last_reply_at?",
    p.is_pinned as pinned,
    p.is_locked as locked,
    pr.user_id is not null as read_by_me
from posts p
join users u on u.user_id = p.author_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
left join lateral (
    select
        cu.nickname as author_name,
        cu.avatar_url as author_avatar_url,
        lc.created_at as replied_at
    from comments lc
    join users cu on cu.user_id = lc.author_id
    where lc.post_id = p.post_id
      and lc.status = 'visible'
    order by lc.created_at desc, lc.comment_id desc
    limit 1
) lr on true
left join post_reads pr on pr.post_id = p.post_id and pr.user_id = $3
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
    lr.author_name,
    lr.author_avatar_url,
    lr.replied_at,
    p.is_pinned,
    p.is_locked,
    pr.user_id,
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
    p.status,
    p.is_pinned as "pinned!",
    p.is_locked as "locked!"
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
    p.status,
    p.is_pinned,
    p.is_locked
limit 1
"#
    }

    pub async fn list_published_summaries(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<Vec<PostSummary>> {
        Self::list_published_summaries_for_user(pool, limit, offset, None).await
    }

    pub async fn list_published_summaries_for_user(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
        current_user_id: Option<Uuid>,
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
    p.published_at,
    lr.author_name as "last_reply_author_name?",
    lr.author_avatar_url as "last_reply_author_avatar_url?",
    lr.replied_at as "last_reply_at?",
    p.is_pinned as "pinned!",
    p.is_locked as "locked!",
    pr.user_id is not null as "read_by_me!"
from posts p
join users u on u.user_id = p.author_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
left join lateral (
    select
        cu.nickname as author_name,
        cu.avatar_url as author_avatar_url,
        lc.created_at as replied_at
    from comments lc
    join users cu on cu.user_id = lc.author_id
    where lc.post_id = p.post_id
      and lc.status = 'visible'
    order by lc.created_at desc, lc.comment_id desc
    limit 1
) lr on true
left join post_reads pr on pr.post_id = p.post_id and pr.user_id = $3
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
    lr.author_name,
    lr.author_avatar_url,
    lr.replied_at,
    p.is_pinned,
    p.is_locked,
    pr.user_id,
    p.created_at
order by p.is_pinned desc, p.published_at desc nulls last, p.created_at desc
limit $1 offset $2
"#,
            limit,
            offset,
            current_user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(PostSummary::from).collect())
    }

    pub async fn list_homepage_summaries(
        pool: &sqlx::PgPool,
        query: &HomeQuery,
        current_user_id: Option<Uuid>,
        prefer_recommended: bool,
    ) -> sqlx::Result<Vec<PostSummary>> {
        let query = query.clone().normalized();
        let limit = query.page_size as i64;
        let offset = ((query.page.saturating_sub(1)) * query.page_size) as i64;
        let category = query.category.as_deref();
        let tag = query.tag.as_deref();
        let tab = match query.tab {
            HomeTab::Hot => "hot",
            HomeTab::Unanswered => "unanswered",
            HomeTab::Following => "following",
            HomeTab::Latest => "latest",
        };
        let sort = match (query.tab, query.sort) {
            (HomeTab::Hot, _) | (_, HomeSort::Hot) => "hot",
            (_, HomeSort::Created) => "created",
            (_, HomeSort::Replies) => "replies",
            (_, HomeSort::Views) => "views",
            (_, HomeSort::LastReply) => "last_reply",
        };
        let since = match query.time {
            HomeTimeRange::All => None,
            HomeTimeRange::Today => Some(OffsetDateTime::now_utc() - time::Duration::days(1)),
            HomeTimeRange::Week => Some(OffsetDateTime::now_utc() - time::Duration::weeks(1)),
            HomeTimeRange::Month => Some(OffsetDateTime::now_utc() - time::Duration::days(30)),
        };

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
    p.published_at,
    lr.author_name as "last_reply_author_name?",
    lr.author_avatar_url as "last_reply_author_avatar_url?",
    lr.replied_at as "last_reply_at?",
    p.is_pinned as "pinned!",
    p.is_locked as "locked!",
    pr.user_id is not null as "read_by_me!"
from posts p
join users u on u.user_id = p.author_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
left join lateral (
    select
        cu.nickname as author_name,
        cu.avatar_url as author_avatar_url,
        lc.created_at as replied_at
    from comments lc
    join users cu on cu.user_id = lc.author_id
    where lc.post_id = p.post_id
      and lc.status = 'visible'
    order by lc.created_at desc, lc.comment_id desc
    limit 1
) lr on true
left join post_reads pr on pr.post_id = p.post_id and pr.user_id = $3
where p.status = 'published'
  and ($4::text is null or c.name = $4)
  and ($5::text is null or exists (
      select 1
      from post_tags fpt
      join tags ft on ft.tag_id = fpt.tag_id
      where fpt.post_id = p.post_id
        and lower(ft.name) = lower($5)
  ))
  and ($6::text <> 'unanswered' or p.comment_count = 0)
  and ($6::text <> 'following' or exists (
      select 1
      from follows ff
      where ff.follower_id = $3
        and ff.followee_id = p.author_id
  ))
  and ($7::timestamptz is null or p.published_at >= $7)
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
    lr.author_name,
    lr.author_avatar_url,
    lr.replied_at,
    p.is_pinned,
    p.is_locked,
    p.is_recommended,
    pr.user_id,
    p.created_at
order by
    case when $9 then p.is_recommended else false end desc,
    p.is_pinned desc,
    case when $8 = 'hot' then p.view_count + p.comment_count * 20 + p.like_count * 10 + p.favorite_count * 5 end desc nulls last,
    case when $8 = 'replies' then p.comment_count end desc nulls last,
    case when $8 = 'views' then p.view_count end desc nulls last,
    case when $8 = 'created' then p.created_at end desc nulls last,
    case when $8 = 'last_reply' then coalesce(lr.replied_at, p.published_at, p.created_at) end desc nulls last,
    coalesce(p.published_at, p.created_at) desc,
    p.created_at desc
limit $1 offset $2
"#,
            limit,
            offset,
            current_user_id,
            category,
            tag,
            tab,
            since,
            sort,
            prefer_recommended
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(PostSummary::from).collect())
    }

    pub async fn count_homepage_summaries(
        pool: &sqlx::PgPool,
        query: &HomeQuery,
        current_user_id: Option<Uuid>,
    ) -> sqlx::Result<i64> {
        let query = query.clone().normalized();
        let category = query.category.as_deref();
        let tag = query.tag.as_deref();
        let tab = match query.tab {
            HomeTab::Hot => "hot",
            HomeTab::Unanswered => "unanswered",
            HomeTab::Following => "following",
            HomeTab::Latest => "latest",
        };
        let since = match query.time {
            HomeTimeRange::All => None,
            HomeTimeRange::Today => Some(OffsetDateTime::now_utc() - time::Duration::days(1)),
            HomeTimeRange::Week => Some(OffsetDateTime::now_utc() - time::Duration::weeks(1)),
            HomeTimeRange::Month => Some(OffsetDateTime::now_utc() - time::Duration::days(30)),
        };

        let row = sqlx::query!(
            r#"
select count(distinct p.post_id) as "total!"
from posts p
left join categories c on c.category_id = p.category_id
where p.status = 'published'
  and ($2::text is null or c.name = $2)
  and ($3::text is null or exists (
      select 1
      from post_tags fpt
      join tags ft on ft.tag_id = fpt.tag_id
      where fpt.post_id = p.post_id
        and lower(ft.name) = lower($3)
  ))
  and ($4::text <> 'unanswered' or p.comment_count = 0)
  and ($4::text <> 'following' or exists (
      select 1
      from follows ff
      where ff.follower_id = $1
        and ff.followee_id = p.author_id
  ))
  and ($5::timestamptz is null or p.published_at >= $5)
"#,
            current_user_id,
            category,
            tag,
            tab,
            since
        )
        .fetch_one(pool)
        .await?;

        Ok(row.total)
    }

    pub async fn list_related_summaries(
        pool: &sqlx::PgPool,
        post_id: Uuid,
        limit: i64,
        current_user_id: Option<Uuid>,
    ) -> sqlx::Result<Vec<PostSummary>> {
        let rows = sqlx::query_as!(
            PostSummaryRow,
            r#"
with source_post as (
    select post_id, category_id
    from posts
    where post_id = $1
    limit 1
),
source_tags as (
    select tag_id
    from post_tags
    where post_id = $1
)
select
    p.post_id,
    p.title,
    p.summary,
    p.author_id,
    u.nickname as author_name,
    u.avatar_url as author_avatar_url,
    c.name as category_name,
    coalesce(array_remove(array_agg(distinct t.name), null), array[]::text[]) as "tags!: Vec<String>",
    p.view_count,
    p.comment_count,
    p.like_count,
    p.favorite_count,
    p.published_at,
    lr.author_name as "last_reply_author_name?",
    lr.author_avatar_url as "last_reply_author_avatar_url?",
    lr.replied_at as "last_reply_at?",
    p.is_pinned as "pinned!",
    p.is_locked as "locked!",
    pr.user_id is not null as "read_by_me!"
from source_post sp
join posts p on p.post_id <> sp.post_id
join users u on u.user_id = p.author_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
left join post_tags shared_pt on shared_pt.post_id = p.post_id
    and shared_pt.tag_id in (select tag_id from source_tags)
left join lateral (
    select
        cu.nickname as author_name,
        cu.avatar_url as author_avatar_url,
        lc.created_at as replied_at
    from comments lc
    join users cu on cu.user_id = lc.author_id
    where lc.post_id = p.post_id
      and lc.status = 'visible'
    order by lc.created_at desc, lc.comment_id desc
    limit 1
) lr on true
left join post_reads pr on pr.post_id = p.post_id and pr.user_id = $3
where p.status = 'published'
  and (
      shared_pt.tag_id is not null
      or (sp.category_id is not null and p.category_id = sp.category_id)
  )
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
    lr.author_name,
    lr.author_avatar_url,
    lr.replied_at,
    p.is_pinned,
    p.is_locked,
    pr.user_id,
    p.created_at
order by
    count(distinct shared_pt.tag_id) desc,
    p.comment_count desc,
    p.view_count desc,
    coalesce(p.published_at, p.created_at) desc
limit $2
"#,
            post_id,
            limit,
            current_user_id
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
    p.status,
    p.is_pinned as "pinned!",
    p.is_locked as "locked!"
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
    p.status,
    p.is_pinned,
    p.is_locked
limit 1
"#,
            post_id
        )
        .fetch_optional(pool)
            .await?;

        Ok(row.map(PostDetail::from))
    }

    pub async fn mark_post_read(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        post_id: Uuid,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
insert into post_reads (
    user_id,
    post_id,
    read_at
)
values ($1, $2, now())
on conflict (user_id, post_id) do update
set read_at = excluded.read_at
"#,
            user_id,
            post_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn increment_view_count(
        pool: &sqlx::PgPool,
        post_id: Uuid,
    ) -> sqlx::Result<Option<i64>> {
        let row = sqlx::query!(
            r#"
update posts
set view_count = view_count + 1,
    updated_at = now()
where post_id = $1
  and status = 'published'
returning view_count
"#,
            post_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| row.view_count))
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
    published_at,
    is_pinned,
    is_locked
)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
            detail.summary.published_at,
            detail.summary.pinned,
            detail.locked
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
