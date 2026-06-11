use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportTargetType {
    Post,
    Comment,
    User,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Pending,
    Handled,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateReportRequest {
    pub target_type: ReportTargetType,
    pub target_id: Uuid,
    pub reason: String,
    pub description: Option<String>,
}

impl CreateReportRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.reason.trim().is_empty() {
            return Err("举报原因不能为空".to_string());
        }
        if self.reason.chars().count() > 120 {
            return Err("举报原因不能超过 120 个字符".to_string());
        }
        if self
            .description
            .as_deref()
            .is_some_and(|value| value.chars().count() > 500)
        {
            return Err("补充说明不能超过 500 个字符".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HandleReportRequest {
    pub status: ReportStatus,
    pub note: String,
}

impl HandleReportRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.status == ReportStatus::Pending {
            return Err("处理结果不能回退为待处理".to_string());
        }
        if self.note.trim().is_empty() {
            return Err("处理说明不能为空".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReportItem {
    pub report_id: Uuid,
    pub target_type: ReportTargetType,
    pub target_id: Uuid,
    pub target_title: Option<String>,
    pub reporter_id: Uuid,
    pub reporter_name: String,
    pub reason: String,
    pub description: Option<String>,
    pub status: ReportStatus,
    pub handler_id: Option<Uuid>,
    pub handler_name: Option<String>,
    pub handle_note: Option<String>,
    pub created_at: OffsetDateTime,
    pub handled_at: Option<OffsetDateTime>,
}
