use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum FilePurpose {
    Avatar,
    PostCover,
    PostImage,
    AnnouncementImage,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FileUploadRequest {
    pub original_filename: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub sha256: String,
    pub purpose: FilePurpose,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StoredFile {
    pub file_id: Uuid,
    pub original_filename: String,
    pub bucket: String,
    pub object_key: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub sha256: String,
    pub uploaded_by: Uuid,
    pub public_url: String,
    pub uploaded_at: OffsetDateTime,
}
