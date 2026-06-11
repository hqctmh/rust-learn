use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementStatus {
    Draft,
    Published,
    Withdrawn,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementAudience {
    AllUsers,
    UserIds(Vec<Uuid>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateAnnouncementRequest {
    pub title: String,
    pub content: String,
    pub announcement_type: String,
    pub pinned: bool,
    pub effective_at: Option<OffsetDateTime>,
    pub expires_at: Option<OffsetDateTime>,
    pub audience: AnnouncementAudience,
}

impl CreateAnnouncementRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("公告标题不能为空".to_string());
        }
        if self.content.trim().is_empty() {
            return Err("公告内容不能为空".to_string());
        }
        if self.announcement_type.trim().is_empty() {
            return Err("公告类型不能为空".to_string());
        }
        if self
            .expires_at
            .zip(self.effective_at)
            .is_some_and(|(expires_at, effective_at)| expires_at <= effective_at)
        {
            return Err("失效时间必须晚于生效时间".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnnouncementItem {
    pub announcement_id: Uuid,
    pub title: String,
    pub content: String,
    pub announcement_type: String,
    pub pinned: bool,
    pub status: AnnouncementStatus,
    pub audience: AnnouncementAudience,
    pub creator_id: Uuid,
    pub creator_name: String,
    pub effective_at: Option<OffsetDateTime>,
    pub expires_at: Option<OffsetDateTime>,
    pub published_at: Option<OffsetDateTime>,
    pub withdrawn_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl AnnouncementItem {
    pub fn is_public_at(&self, now: OffsetDateTime) -> bool {
        self.status == AnnouncementStatus::Published
            && self
                .effective_at
                .is_none_or(|effective_at| effective_at <= now)
            && self.expires_at.is_none_or(|expires_at| expires_at > now)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnnouncementReadState {
    pub announcement_id: Uuid,
    pub user_id: Uuid,
    pub read: bool,
}
