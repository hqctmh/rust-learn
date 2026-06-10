use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ReportTarget {
    Post(Uuid),
    Comment(Uuid),
    User(Uuid),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateReportRequest {
    pub reporter_id: Uuid,
    pub target: ReportTarget,
    pub reason: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ReportStatus {
    Open,
    Resolved,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Report {
    pub report_id: Uuid,
    pub reporter_id: Uuid,
    pub target: ReportTarget,
    pub reason: String,
    pub note: Option<String>,
    pub status: ReportStatus,
    pub handled_by: Option<Uuid>,
    pub handled_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ModerationAction {
    TakePostOffline,
    DeleteComment,
    DisableUser,
    NoAction,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ReportDecision {
    Resolved { action: ModerationAction },
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AuditLogEntry {
    pub audit_id: Uuid,
    pub actor_id: Uuid,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminStats {
    pub user_total: usize,
    pub post_total: usize,
    pub comment_total: usize,
    pub like_total: usize,
    pub favorite_total: usize,
    pub open_report_total: usize,
    pub audit_log_total: usize,
    pub notification_total: usize,
    pub file_total: usize,
}
