use crate::domain::home::{HomeActiveAuthor, HomeAnnouncement, HomeCategory, HomeTag};

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct HomeCategoryRow {
    pub name: String,
    pub count: i64,
    pub color: String,
}

impl From<HomeCategoryRow> for HomeCategory {
    fn from(row: HomeCategoryRow) -> Self {
        Self {
            name: row.name,
            count: row.count.max(0) as u32,
            color: row.color,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct HomeTagRow {
    pub name: String,
    pub count: i64,
}

impl From<HomeTagRow> for HomeTag {
    fn from(row: HomeTagRow) -> Self {
        Self {
            name: row.name,
            count: row.count.max(0) as u32,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct HomeAnnouncementRow {
    pub title: String,
    pub date_label: String,
}

impl From<HomeAnnouncementRow> for HomeAnnouncement {
    fn from(row: HomeAnnouncementRow) -> Self {
        Self {
            title: row.title,
            date_label: row.date_label,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct HomeActiveAuthorRow {
    pub name: String,
    pub avatar_label: String,
    pub reply_count_label: String,
}

impl From<HomeActiveAuthorRow> for HomeActiveAuthor {
    fn from(row: HomeActiveAuthorRow) -> Self {
        Self {
            name: row.name,
            avatar_label: row.avatar_label,
            reply_count_label: row.reply_count_label,
        }
    }
}

pub struct PostgresHomeRepository;

impl PostgresHomeRepository {
    pub fn homepage_categories_sql() -> &'static str {
        r#"
select
    c.name,
    count(p.post_id) as count,
    case c.name
        when '公告' then 'blue'
        when '教程' then 'green'
        when '问题' then 'orange'
        when '经验分享' then 'sky'
        when '站务' then 'purple'
        else 'gray'
    end as color
from categories c
left join posts p on p.category_id = c.category_id and p.status = 'published'
where c.enabled = true
group by c.category_id, c.name, c.sort_order
order by c.sort_order asc, c.name asc
"#
    }

    pub fn hot_tags_sql() -> &'static str {
        r#"
select
    t.name,
    greatest(t.use_count, count(p.post_id)) as count
from tags t
left join post_tags pt on pt.tag_id = t.tag_id
left join posts p on p.post_id = pt.post_id
where p.status = 'published'
  and t.enabled = true
  or (t.enabled = true and t.use_count > 0 and pt.post_id is null)
group by t.tag_id, t.name, t.sort_order, t.use_count
order by greatest(t.use_count, count(p.post_id)) desc, t.sort_order asc, t.name asc
limit $1
"#
    }

    pub fn announcements_sql() -> &'static str {
        r#"
select
    title,
    to_char(coalesce(starts_at, created_at), 'FMMM 月 FMDD 日') as date_label
from announcements
where status = 'published'
  and (starts_at is null or starts_at <= now())
  and (ends_at is null or ends_at > now())
order by is_pinned desc, created_at desc
limit $1
"#
    }

    pub fn active_authors_sql() -> &'static str {
        r#"
select
    u.nickname as name,
    left(u.nickname, 1) as avatar_label,
    case
        when count(c.comment_id) >= 1000
            then trim(to_char(count(c.comment_id)::numeric / 1000.0, 'FM999999990.0')) || 'k 条回复'
        else count(c.comment_id)::text || ' 条回复'
    end as reply_count_label
from users u
join comments c on c.author_id = u.user_id and c.status = 'visible'
group by u.user_id, u.nickname
order by count(c.comment_id) desc, u.nickname asc
limit $1
"#
    }

    pub async fn list_homepage_categories(pool: &sqlx::PgPool) -> sqlx::Result<Vec<HomeCategory>> {
        let rows = sqlx::query_as!(
            HomeCategoryRow,
            r#"
select
    c.name,
    count(p.post_id) as "count!",
    case c.name
        when '公告' then 'blue'
        when '教程' then 'green'
        when '问题' then 'orange'
        when '经验分享' then 'sky'
        when '站务' then 'purple'
        else 'gray'
    end as "color!"
from categories c
left join posts p on p.category_id = c.category_id and p.status = 'published'
where c.enabled = true
group by c.category_id, c.name, c.sort_order
order by c.sort_order asc, c.name asc
"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(HomeCategory::from).collect())
    }

    pub async fn list_hot_tags(pool: &sqlx::PgPool, limit: i64) -> sqlx::Result<Vec<HomeTag>> {
        let rows = sqlx::query_as!(
            HomeTagRow,
            r#"
select
    t.name,
    greatest(t.use_count, count(p.post_id)) as "count!"
from tags t
left join post_tags pt on pt.tag_id = t.tag_id
left join posts p on p.post_id = pt.post_id
where p.status = 'published'
  and t.enabled = true
  or (t.enabled = true and t.use_count > 0 and pt.post_id is null)
group by t.tag_id, t.name, t.sort_order, t.use_count
order by greatest(t.use_count, count(p.post_id)) desc, t.sort_order asc, t.name asc
limit $1
"#,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(HomeTag::from).collect())
    }

    pub async fn list_announcements(
        pool: &sqlx::PgPool,
        limit: i64,
    ) -> sqlx::Result<Vec<HomeAnnouncement>> {
        let rows = sqlx::query_as!(
            HomeAnnouncementRow,
            r#"
select
    title,
    to_char(coalesce(starts_at, created_at), 'FMMM 月 FMDD 日') as "date_label!"
from announcements
where status = 'published'
  and (starts_at is null or starts_at <= now())
  and (ends_at is null or ends_at > now())
order by is_pinned desc, created_at desc
limit $1
"#,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(HomeAnnouncement::from).collect())
    }

    pub async fn list_active_authors(
        pool: &sqlx::PgPool,
        limit: i64,
    ) -> sqlx::Result<Vec<HomeActiveAuthor>> {
        let rows = sqlx::query_as!(
            HomeActiveAuthorRow,
            r#"
select
    u.nickname as name,
    left(u.nickname, 1) as "avatar_label!",
    case
        when count(c.comment_id) >= 1000
            then trim(to_char(count(c.comment_id)::numeric / 1000.0, 'FM999999990.0')) || 'k 条回复'
        else count(c.comment_id)::text || ' 条回复'
    end as "reply_count_label!"
from users u
join comments c on c.author_id = u.user_id and c.status = 'visible'
group by u.user_id, u.nickname
order by count(c.comment_id) desc, u.nickname asc
limit $1
"#,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(HomeActiveAuthor::from).collect())
    }
}
