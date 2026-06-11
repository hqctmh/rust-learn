use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub const MAX_IMAGE_SIZE_BYTES: i64 = 5 * 1024 * 1024;
pub const ASSET_BUCKET: &str = "post-assets";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileUsage {
    Avatar,
    CoverImage,
    MarkdownImage,
    AnnouncementImage,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FileUploadRequest {
    pub original_filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub content_hash: String,
    pub usage: FileUsage,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FileAsset {
    pub file_id: Uuid,
    pub original_filename: String,
    pub bucket: String,
    pub storage_key: String,
    pub file_size: i64,
    pub mime_type: String,
    pub file_hash: String,
    pub uploader_id: Uuid,
    pub public_url: String,
    pub markdown_image: String,
    pub uploaded_at: OffsetDateTime,
}

impl FileUploadRequest {
    pub fn validate(&self) -> Result<(), String> {
        let filename = self.original_filename.trim();
        if filename.is_empty() {
            return Err("文件名不能为空".to_string());
        }
        if self.file_size <= 0 {
            return Err("文件大小必须大于 0".to_string());
        }
        if self.file_size > MAX_IMAGE_SIZE_BYTES {
            return Err(format!(
                "图片大小不能超过 {}MB",
                MAX_IMAGE_SIZE_BYTES / 1024 / 1024
            ));
        }
        if !is_allowed_image_mime(&self.mime_type) {
            return Err("仅支持 PNG、JPEG、WebP 图片".to_string());
        }
        if self.content_hash.trim().is_empty() {
            return Err("文件 hash 不能为空".to_string());
        }
        Ok(())
    }
}

pub fn build_file_asset(file_id: Uuid, uploader_id: Uuid, request: FileUploadRequest) -> FileAsset {
    let filename = sanitize_filename(&request.original_filename);
    let usage_path = match request.usage {
        FileUsage::Avatar => "avatars",
        FileUsage::CoverImage => "covers",
        FileUsage::MarkdownImage => "markdown",
        FileUsage::AnnouncementImage => "announcements",
    };
    let hash_prefix = request
        .content_hash
        .chars()
        .filter(|value| value.is_ascii_alphanumeric())
        .take(12)
        .collect::<String>();
    let storage_key = format!("{usage_path}/{hash_prefix}/{filename}");
    let public_url = format!("/uploads/{storage_key}");

    FileAsset {
        file_id,
        original_filename: filename.clone(),
        bucket: ASSET_BUCKET.to_string(),
        storage_key,
        file_size: request.file_size,
        mime_type: request.mime_type,
        file_hash: request.content_hash,
        uploader_id,
        markdown_image: format!("![{filename}]({public_url})"),
        public_url,
        uploaded_at: OffsetDateTime::now_utc(),
    }
}

pub fn is_allowed_image_mime(mime_type: &str) -> bool {
    matches!(mime_type, "image/png" | "image/jpeg" | "image/webp")
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .trim()
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() || matches!(value, '.' | '-' | '_') {
                value
            } else {
                '-'
            }
        })
        .collect()
}
