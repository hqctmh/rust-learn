use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Conversaction {
    pub id: Uuid,
    pub doc_id: String,
    pub doc_type: String,
    pub user_id: i64,
    pub title :String,
    pub r#type:String,
    pub inline_type:String,
    pub created_at:DateTime<Utc>,
    pub updated_at
}
