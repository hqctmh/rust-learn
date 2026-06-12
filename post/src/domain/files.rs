use base64::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
pub struct FileBinaryUploadRequest {
    pub original_filename: String,
    pub mime_type: String,
    pub content_base64: String,
    pub usage: FileUsage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileObjectUpload {
    pub bucket: String,
    pub storage_key: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
    pub asset: FileUploadRequest,
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

impl FileBinaryUploadRequest {
    pub fn decoded_bytes(&self) -> Result<Vec<u8>, String> {
        BASE64_STANDARD
            .decode(self.content_base64.trim())
            .map_err(|_| "图片内容不是有效的 base64".to_string())
    }

    pub fn to_upload_request(&self) -> Result<FileUploadRequest, String> {
        let bytes = self.decoded_bytes()?;
        self.to_upload_request_from_bytes(&bytes)
    }

    pub fn to_object_upload(&self) -> Result<FileObjectUpload, String> {
        let bytes = self.decoded_bytes()?;
        let asset = self.to_upload_request_from_bytes(&bytes)?;
        let storage_key = build_storage_key(&asset);
        Ok(FileObjectUpload {
            bucket: ASSET_BUCKET.to_string(),
            storage_key,
            content_type: asset.mime_type.clone(),
            bytes,
            asset,
        })
    }

    fn to_upload_request_from_bytes(&self, bytes: &[u8]) -> Result<FileUploadRequest, String> {
        let file_size = i64::try_from(bytes.len()).map_err(|_| "文件大小超出范围".to_string())?;
        let request = FileUploadRequest {
            original_filename: self.original_filename.clone(),
            file_size,
            mime_type: self.mime_type.clone(),
            content_hash: sha256_hex(bytes),
            usage: self.usage.clone(),
        };
        request.validate()?;
        Ok(request)
    }
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
    let storage_key = build_storage_key(&request);
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

fn build_storage_key(request: &FileUploadRequest) -> String {
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
    format!("{usage_path}/{hash_prefix}/{filename}")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
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
