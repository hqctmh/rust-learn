use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::announcements::{AnnouncementItem, AnnouncementStatus, CreateAnnouncementRequest},
    error::ForumError,
};

pub struct AnnouncementService;

impl AnnouncementService {
    pub fn build_draft(
        announcement_id: Uuid,
        creator_id: Uuid,
        creator_name: &str,
        request: CreateAnnouncementRequest,
        now: OffsetDateTime,
    ) -> Result<AnnouncementItem, ForumError> {
        request.validate().map_err(ForumError::Validation)?;

        Ok(AnnouncementItem {
            announcement_id,
            title: request.title.trim().to_string(),
            content: request.content.trim().to_string(),
            announcement_type: request.announcement_type.trim().to_string(),
            pinned: request.pinned,
            status: AnnouncementStatus::Draft,
            audience: request.audience,
            creator_id,
            creator_name: creator_name.to_string(),
            effective_at: request.effective_at,
            expires_at: request.expires_at,
            published_at: None,
            withdrawn_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn publish(
        announcement: &mut AnnouncementItem,
        now: OffsetDateTime,
    ) -> Result<(), ForumError> {
        announcement.status = AnnouncementStatus::Published;
        announcement.published_at = Some(now);
        announcement.withdrawn_at = None;
        announcement.updated_at = now;
        Ok(())
    }

    pub fn withdraw(
        announcement: &mut AnnouncementItem,
        now: OffsetDateTime,
    ) -> Result<(), ForumError> {
        announcement.status = AnnouncementStatus::Withdrawn;
        announcement.withdrawn_at = Some(now);
        announcement.updated_at = now;
        Ok(())
    }

    pub fn notification_body(content: &str) -> String {
        content.chars().take(120).collect()
    }
}
