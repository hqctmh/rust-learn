use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::reports::{ReportItem, ReportStatus, ReportTargetType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportRow {
    pub report_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub target_title: Option<String>,
    pub reporter_id: Uuid,
    pub reporter_name: String,
    pub reason: String,
    pub description: String,
    pub status: String,
    pub handler_id: Option<Uuid>,
    pub handler_name: Option<String>,
    pub handle_note: String,
    pub created_at: OffsetDateTime,
    pub handled_at: Option<OffsetDateTime>,
}

impl ReportRow {
    pub fn into_item(self) -> ReportItem {
        ReportItem {
            report_id: self.report_id,
            target_type: report_target_type_from_str(&self.target_type),
            target_id: self.target_id,
            target_title: self.target_title,
            reporter_id: self.reporter_id,
            reporter_name: self.reporter_name,
            reason: self.reason,
            description: non_empty_string(self.description),
            status: report_status_from_str(&self.status),
            handler_id: self.handler_id,
            handler_name: self.handler_name,
            handle_note: non_empty_string(self.handle_note),
            created_at: self.created_at,
            handled_at: self.handled_at,
        }
    }
}

pub struct PostgresReportRepository;

impl PostgresReportRepository {
    pub fn list_reports_sql() -> &'static str {
        r#"
select
    r.report_id,
    r.target_type,
    r.target_id,
    case
        when r.target_type = 'post' then p.title
        when r.target_type = 'comment' then left(c.content, 40)
        when r.target_type = 'user' then tu.nickname
        else null
    end as target_title,
    r.reporter_id,
    ru.nickname as reporter_name,
    r.reason,
    r.description,
    r.status,
    r.handler_id,
    hu.nickname as handler_name,
    r.handle_note,
    r.created_at,
    r.handled_at
from reports r
join users ru on ru.user_id = r.reporter_id
left join users hu on hu.user_id = r.handler_id
left join posts p on r.target_type = 'post' and p.post_id = r.target_id
left join comments c on r.target_type = 'comment' and c.comment_id = r.target_id
left join users tu on r.target_type = 'user' and tu.user_id = r.target_id
order by r.created_at desc
"#
    }

    pub async fn insert_report(pool: &sqlx::PgPool, report: &ReportItem) -> sqlx::Result<()> {
        let target_type = report_target_type_as_str(&report.target_type);
        let status = report_status_as_str(&report.status);
        let description = report.description.clone().unwrap_or_default();

        sqlx::query!(
            r#"
insert into reports (
    report_id,
    reporter_id,
    target_type,
    target_id,
    reason,
    description,
    status
)
values ($1, $2, $3, $4, $5, $6, $7)
"#,
            report.report_id,
            report.reporter_id,
            target_type,
            report.target_id,
            report.reason,
            description,
            status
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list_reports(pool: &sqlx::PgPool) -> sqlx::Result<Vec<ReportItem>> {
        let rows = sqlx::query_as!(
            ReportRow,
            r#"
select
    r.report_id,
    r.target_type,
    r.target_id,
    case
        when r.target_type = 'post' then p.title
        when r.target_type = 'comment' then left(c.content, 40)
        when r.target_type = 'user' then tu.nickname
        else null
    end as "target_title?",
    r.reporter_id,
    ru.nickname as reporter_name,
    r.reason,
    r.description,
    r.status,
    r.handler_id,
    hu.nickname as "handler_name?",
    r.handle_note,
    r.created_at,
    r.handled_at
from reports r
join users ru on ru.user_id = r.reporter_id
left join users hu on hu.user_id = r.handler_id
left join posts p on r.target_type = 'post' and p.post_id = r.target_id
left join comments c on r.target_type = 'comment' and c.comment_id = r.target_id
left join users tu on r.target_type = 'user' and tu.user_id = r.target_id
order by r.created_at desc
"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(ReportRow::into_item).collect())
    }

    pub async fn find_report(
        pool: &sqlx::PgPool,
        report_id: Uuid,
    ) -> sqlx::Result<Option<ReportItem>> {
        let row = sqlx::query_as!(
            ReportRow,
            r#"
select
    r.report_id,
    r.target_type,
    r.target_id,
    case
        when r.target_type = 'post' then p.title
        when r.target_type = 'comment' then left(c.content, 40)
        when r.target_type = 'user' then tu.nickname
        else null
    end as "target_title?",
    r.reporter_id,
    ru.nickname as reporter_name,
    r.reason,
    r.description,
    r.status,
    r.handler_id,
    hu.nickname as "handler_name?",
    r.handle_note,
    r.created_at,
    r.handled_at
from reports r
join users ru on ru.user_id = r.reporter_id
left join users hu on hu.user_id = r.handler_id
left join posts p on r.target_type = 'post' and p.post_id = r.target_id
left join comments c on r.target_type = 'comment' and c.comment_id = r.target_id
left join users tu on r.target_type = 'user' and tu.user_id = r.target_id
where r.report_id = $1
limit 1
"#,
            report_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ReportRow::into_item))
    }

    pub async fn update_report_handle(
        pool: &sqlx::PgPool,
        report: &ReportItem,
    ) -> sqlx::Result<u64> {
        let status = report_status_as_str(&report.status);
        let handle_note = report.handle_note.clone().unwrap_or_default();
        let result = sqlx::query!(
            r#"
update reports
set
    status = $2,
    handler_id = $3,
    handle_note = $4,
    handled_at = $5
where report_id = $1
"#,
            report.report_id,
            status,
            report.handler_id,
            handle_note,
            report.handled_at
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

fn report_target_type_as_str(target_type: &ReportTargetType) -> &'static str {
    match target_type {
        ReportTargetType::Post => "post",
        ReportTargetType::Comment => "comment",
        ReportTargetType::User => "user",
    }
}

fn report_target_type_from_str(target_type: &str) -> ReportTargetType {
    match target_type {
        "comment" => ReportTargetType::Comment,
        "user" => ReportTargetType::User,
        _ => ReportTargetType::Post,
    }
}

fn report_status_as_str(status: &ReportStatus) -> &'static str {
    match status {
        ReportStatus::Pending => "pending",
        ReportStatus::Handled => "accepted",
        ReportStatus::Rejected => "rejected",
    }
}

fn report_status_from_str(status: &str) -> ReportStatus {
    match status {
        "accepted" | "handled" => ReportStatus::Handled,
        "rejected" => ReportStatus::Rejected,
        _ => ReportStatus::Pending,
    }
}

fn non_empty_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}
