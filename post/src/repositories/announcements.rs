use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::announcements::{
    AnnouncementAudience, AnnouncementItem, AnnouncementReadState, AnnouncementStatus,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnouncementRow {
    pub announcement_id: Uuid,
    pub title: String,
    pub content: String,
    pub announcement_type: String,
    pub pinned: bool,
    pub status: String,
    pub audience_type: String,
    pub audience_user_ids: Vec<Uuid>,
    pub creator_id: Uuid,
    pub creator_name: String,
    pub effective_at: Option<OffsetDateTime>,
    pub expires_at: Option<OffsetDateTime>,
    pub published_at: Option<OffsetDateTime>,
    pub withdrawn_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl AnnouncementRow {
    pub fn into_item(self) -> AnnouncementItem {
        AnnouncementItem {
            announcement_id: self.announcement_id,
            title: self.title,
            content: self.content,
            announcement_type: self.announcement_type,
            pinned: self.pinned,
            status: announcement_status_from_str(&self.status),
            audience: announcement_audience_from_parts(&self.audience_type, self.audience_user_ids),
            creator_id: self.creator_id,
            creator_name: self.creator_name,
            effective_at: self.effective_at,
            expires_at: self.expires_at,
            published_at: self.published_at,
            withdrawn_at: self.withdrawn_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

pub struct PostgresAnnouncementRepository;

impl PostgresAnnouncementRepository {
    pub fn list_admin_announcements_sql() -> &'static str {
        r#"
select
    a.announcement_id,
    a.title,
    a.content,
    a.announcement_type,
    a.is_pinned as pinned,
    a.status,
    a.audience_type,
    a.audience_user_ids,
    a.creator_id,
    u.nickname as creator_name,
    a.starts_at as effective_at,
    a.ends_at as expires_at,
    a.published_at,
    a.withdrawn_at,
    a.created_at,
    a.updated_at
from announcements a
join users u on u.user_id = a.creator_id
order by a.is_pinned desc, a.updated_at desc
"#
    }

    pub async fn insert_announcement(
        pool: &sqlx::PgPool,
        announcement: &AnnouncementItem,
    ) -> sqlx::Result<()> {
        let status = announcement_status_as_str(&announcement.status);
        let (audience_type, audience_user_ids) =
            announcement_audience_parts(&announcement.audience);

        sqlx::query!(
            r#"
insert into announcements (
    announcement_id,
    title,
    content,
    announcement_type,
    is_pinned,
    status,
    starts_at,
    ends_at,
    audience_type,
    audience_user_ids,
    creator_id,
    published_at,
    withdrawn_at,
    created_at,
    updated_at
)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
"#,
            announcement.announcement_id,
            announcement.title,
            announcement.content,
            announcement.announcement_type,
            announcement.pinned,
            status,
            announcement.effective_at,
            announcement.expires_at,
            audience_type,
            &audience_user_ids,
            announcement.creator_id,
            announcement.published_at,
            announcement.withdrawn_at,
            announcement.created_at,
            announcement.updated_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list_admin_announcements(
        pool: &sqlx::PgPool,
    ) -> sqlx::Result<Vec<AnnouncementItem>> {
        let rows = sqlx::query_as!(
            AnnouncementRow,
            r#"
select
    a.announcement_id,
    a.title,
    a.content,
    a.announcement_type,
    a.is_pinned as "pinned!",
    a.status,
    a.audience_type,
    a.audience_user_ids,
    a.creator_id,
    u.nickname as creator_name,
    a.starts_at as "effective_at?",
    a.ends_at as "expires_at?",
    a.published_at,
    a.withdrawn_at,
    a.created_at,
    a.updated_at
from announcements a
join users u on u.user_id = a.creator_id
order by a.is_pinned desc, a.updated_at desc
"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(AnnouncementRow::into_item).collect())
    }

    pub async fn public_announcements(
        pool: &sqlx::PgPool,
        now: OffsetDateTime,
    ) -> sqlx::Result<Vec<AnnouncementItem>> {
        let rows = sqlx::query_as!(
            AnnouncementRow,
            r#"
select
    a.announcement_id,
    a.title,
    a.content,
    a.announcement_type,
    a.is_pinned as "pinned!",
    a.status,
    a.audience_type,
    a.audience_user_ids,
    a.creator_id,
    u.nickname as creator_name,
    a.starts_at as "effective_at?",
    a.ends_at as "expires_at?",
    a.published_at,
    a.withdrawn_at,
    a.created_at,
    a.updated_at
from announcements a
join users u on u.user_id = a.creator_id
where a.status = 'published'
    and (a.starts_at is null or a.starts_at <= $1)
    and (a.ends_at is null or a.ends_at > $1)
order by a.is_pinned desc, a.updated_at desc
"#,
            now
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(AnnouncementRow::into_item).collect())
    }

    pub async fn find_announcement(
        pool: &sqlx::PgPool,
        announcement_id: Uuid,
    ) -> sqlx::Result<Option<AnnouncementItem>> {
        let row = sqlx::query_as!(
            AnnouncementRow,
            r#"
select
    a.announcement_id,
    a.title,
    a.content,
    a.announcement_type,
    a.is_pinned as "pinned!",
    a.status,
    a.audience_type,
    a.audience_user_ids,
    a.creator_id,
    u.nickname as creator_name,
    a.starts_at as "effective_at?",
    a.ends_at as "expires_at?",
    a.published_at,
    a.withdrawn_at,
    a.created_at,
    a.updated_at
from announcements a
join users u on u.user_id = a.creator_id
where a.announcement_id = $1
limit 1
"#,
            announcement_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(AnnouncementRow::into_item))
    }

    pub async fn update_announcement_status(
        pool: &sqlx::PgPool,
        announcement: &AnnouncementItem,
    ) -> sqlx::Result<u64> {
        let status = announcement_status_as_str(&announcement.status);
        let result = sqlx::query!(
            r#"
update announcements
set
    status = $2,
    published_at = $3,
    withdrawn_at = $4,
    updated_at = $5
where announcement_id = $1
"#,
            announcement.announcement_id,
            status,
            announcement.published_at,
            announcement.withdrawn_at,
            announcement.updated_at
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn mark_read(
        pool: &sqlx::PgPool,
        announcement_id: Uuid,
        user_id: Uuid,
    ) -> sqlx::Result<AnnouncementReadState> {
        sqlx::query!(
            r#"
insert into announcement_reads (announcement_id, user_id)
values ($1, $2)
on conflict (announcement_id, user_id) do nothing
"#,
            announcement_id,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(AnnouncementReadState {
            announcement_id,
            user_id,
            read: true,
        })
    }
}

fn announcement_status_as_str(status: &AnnouncementStatus) -> &'static str {
    match status {
        AnnouncementStatus::Draft => "draft",
        AnnouncementStatus::Published => "published",
        AnnouncementStatus::Withdrawn => "offline",
    }
}

fn announcement_status_from_str(status: &str) -> AnnouncementStatus {
    match status {
        "published" => AnnouncementStatus::Published,
        "offline" => AnnouncementStatus::Withdrawn,
        _ => AnnouncementStatus::Draft,
    }
}

fn announcement_audience_parts(audience: &AnnouncementAudience) -> (&'static str, Vec<Uuid>) {
    match audience {
        AnnouncementAudience::AllUsers => ("all_users", Vec::new()),
        AnnouncementAudience::UserIds(user_ids) => ("user_ids", user_ids.clone()),
    }
}

fn announcement_audience_from_parts(
    audience_type: &str,
    audience_user_ids: Vec<Uuid>,
) -> AnnouncementAudience {
    match audience_type {
        "user_ids" => AnnouncementAudience::UserIds(audience_user_ids),
        _ => AnnouncementAudience::AllUsers,
    }
}
