use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    posts::PostSummary,
    users::{UserCommentItem, UserProfile, UserSpace, UserStats},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserProfileRow {
    pub user_id: Uuid,
    pub username: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub bio: String,
    pub registered_at: OffsetDateTime,
}

impl From<UserProfileRow> for UserProfile {
    fn from(row: UserProfileRow) -> Self {
        Self {
            user_id: row.user_id,
            username: row.username,
            nickname: row.nickname,
            avatar_url: row.avatar_url,
            bio: row.bio,
            registered_at: row.registered_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserPostSummaryRow {
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

impl From<UserPostSummaryRow> for PostSummary {
    fn from(row: UserPostSummaryRow) -> Self {
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
            last_reply_author_name: None,
            last_reply_author_avatar_url: None,
            last_reply_at: None,
            pinned: false,
            locked: false,
            read_by_me: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserCommentRow {
    pub post_id: Uuid,
    pub post_title: String,
    pub content: String,
    pub created_at: OffsetDateTime,
}

impl From<UserCommentRow> for UserCommentItem {
    fn from(row: UserCommentRow) -> Self {
        Self {
            post_id: row.post_id,
            post_title: row.post_title,
            content: row.content,
            created_at: row.created_at,
        }
    }
}

pub struct PostgresUserSettingsRepository;

impl PostgresUserSettingsRepository {
    pub async fn update_profile(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        nickname: &str,
        bio: &str,
    ) -> sqlx::Result<Option<UserProfile>> {
        let row = sqlx::query_as!(
            UserProfileRow,
            r#"
update users
set
    nickname = $2,
    bio = $3,
    updated_at = now()
where user_id = $1
  and status = 'active'
returning
    user_id,
    username,
    nickname,
    avatar_url,
    bio,
    created_at as registered_at
"#,
            user_id,
            nickname,
            bio
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(UserProfile::from))
    }

    pub async fn update_avatar(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        avatar_url: &str,
    ) -> sqlx::Result<Option<UserProfile>> {
        let row = sqlx::query_as!(
            UserProfileRow,
            r#"
update users
set
    avatar_url = $2,
    updated_at = now()
where user_id = $1
  and status = 'active'
returning
    user_id,
    username,
    nickname,
    avatar_url,
    bio,
    created_at as registered_at
"#,
            user_id,
            avatar_url
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(UserProfile::from))
    }

    pub async fn update_password(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        new_password: &str,
    ) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            r#"
update users
set
    password_hash = $2,
    updated_at = now()
where user_id = $1
  and status = 'active'
"#,
            user_id,
            new_password
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn user_space(
        pool: &sqlx::PgPool,
        profile_user_id: Uuid,
        viewer_user_id: Option<Uuid>,
    ) -> sqlx::Result<Option<UserSpace>> {
        let Some(profile) = Self::find_profile(pool, profile_user_id).await? else {
            return Ok(None);
        };

        let published_posts = list_user_posts(pool, profile_user_id, "published").await?;
        let draft_posts = list_user_posts(pool, profile_user_id, "draft").await?;
        let comments = list_user_comments(pool, profile_user_id).await?;
        let favorite_posts = list_favorite_posts(pool, profile_user_id).await?;
        let following = list_following_profiles(pool, profile_user_id).await?;
        let followers = list_follower_profiles(pool, profile_user_id).await?;
        let received = sqlx::query!(
            r#"
select
    coalesce(sum(like_count), 0)::bigint as "received_likes!",
    coalesce(sum(favorite_count), 0)::bigint as "received_favorites!"
from posts
where author_id = $1
  and status <> 'deleted'
"#,
            profile_user_id
        )
        .fetch_one(pool)
        .await?;
        let followed_by_viewer = if let Some(viewer_id) = viewer_user_id {
            sqlx::query!(
                r#"
select exists (
    select 1
    from follows
    where follower_id = $1
      and followee_id = $2
) as "exists!"
"#,
                viewer_id,
                profile_user_id
            )
            .fetch_one(pool)
            .await?
            .exists
        } else {
            false
        };

        Ok(Some(UserSpace {
            profile,
            stats: UserStats {
                following: following.len(),
                followers: followers.len(),
                published_posts: published_posts.len(),
                received_likes: received.received_likes,
                received_favorites: received.received_favorites,
            },
            is_me: viewer_user_id == Some(profile_user_id),
            followed_by_viewer,
            published_posts,
            draft_posts,
            comments,
            favorite_posts,
            following,
            followers,
        }))
    }

    async fn find_profile(pool: &sqlx::PgPool, user_id: Uuid) -> sqlx::Result<Option<UserProfile>> {
        let row = sqlx::query_as!(
            UserProfileRow,
            r#"
select
    user_id,
    username,
    nickname,
    avatar_url,
    bio,
    created_at as registered_at
from users
where user_id = $1
  and status = 'active'
limit 1
"#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(UserProfile::from))
    }
}

async fn list_user_posts(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    status: &str,
) -> sqlx::Result<Vec<PostSummary>> {
    let rows = sqlx::query_as!(
        UserPostSummaryRow,
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
where p.author_id = $1
  and p.status = $2
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
    p.updated_at
order by p.updated_at desc
"#,
        user_id,
        status
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(PostSummary::from).collect())
}

async fn list_favorite_posts(pool: &sqlx::PgPool, user_id: Uuid) -> sqlx::Result<Vec<PostSummary>> {
    let rows = sqlx::query_as!(
        UserPostSummaryRow,
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
from post_favorites pf
join posts p on p.post_id = pf.post_id
join users u on u.user_id = p.author_id
left join categories c on c.category_id = p.category_id
left join post_tags pt on pt.post_id = p.post_id
left join tags t on t.tag_id = pt.tag_id
where pf.user_id = $1
  and p.status = 'published'
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
    pf.created_at
order by pf.created_at desc
"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(PostSummary::from).collect())
}

async fn list_user_comments(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> sqlx::Result<Vec<UserCommentItem>> {
    let rows = sqlx::query_as!(
        UserCommentRow,
        r#"
select
    c.post_id,
    p.title as post_title,
    c.content,
    c.created_at
from comments c
join posts p on p.post_id = c.post_id
where c.author_id = $1
order by c.created_at desc
"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(UserCommentItem::from).collect())
}

async fn list_following_profiles(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> sqlx::Result<Vec<UserProfile>> {
    let rows = sqlx::query_as!(
        UserProfileRow,
        r#"
select
    u.user_id,
    u.username,
    u.nickname,
    u.avatar_url,
    u.bio,
    u.created_at as registered_at
from follows f
join users u on u.user_id = f.followee_id
where f.follower_id = $1
  and u.status = 'active'
order by f.created_at desc
"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(UserProfile::from).collect())
}

async fn list_follower_profiles(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> sqlx::Result<Vec<UserProfile>> {
    let rows = sqlx::query_as!(
        UserProfileRow,
        r#"
select
    u.user_id,
    u.username,
    u.nickname,
    u.avatar_url,
    u.bio,
    u.created_at as registered_at
from follows f
join users u on u.user_id = f.follower_id
where f.followee_id = $1
  and u.status = 'active'
order by f.created_at desc
"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(UserProfile::from).collect())
}
