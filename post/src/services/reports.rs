use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::reports::{CreateReportRequest, HandleReportRequest, ReportItem, ReportStatus},
    error::ForumError,
};

pub struct ReportService;

impl ReportService {
    pub fn build_report(
        report_id: Uuid,
        reporter_id: Uuid,
        reporter_name: &str,
        target_title: Option<String>,
        request: CreateReportRequest,
        now: OffsetDateTime,
    ) -> Result<ReportItem, ForumError> {
        request.validate().map_err(ForumError::Validation)?;

        Ok(ReportItem {
            report_id,
            target_type: request.target_type,
            target_id: request.target_id,
            target_title,
            reporter_id,
            reporter_name: reporter_name.to_string(),
            reason: request.reason.trim().to_string(),
            description: request.description.and_then(|value| {
                let value = value.trim().to_string();
                (!value.is_empty()).then_some(value)
            }),
            status: ReportStatus::Pending,
            handler_id: None,
            handler_name: None,
            handle_note: None,
            created_at: now,
            handled_at: None,
        })
    }

    pub fn apply_handle(
        report: &mut ReportItem,
        handler_id: Uuid,
        handler_name: &str,
        request: HandleReportRequest,
        now: OffsetDateTime,
    ) -> Result<(), ForumError> {
        request.validate().map_err(ForumError::Validation)?;

        report.status = request.status;
        report.handler_id = Some(handler_id);
        report.handler_name = Some(handler_name.to_string());
        report.handle_note = Some(request.note.trim().to_string());
        report.handled_at = Some(now);
        Ok(())
    }
}
