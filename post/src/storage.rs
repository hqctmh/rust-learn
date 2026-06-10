use bytes::Bytes;
use uuid::Uuid;

use crate::{
    domain::files::{FilePurpose, FileUploadRequest},
    error::ForumError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectUpload {
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub body: Bytes,
}

impl ObjectUpload {
    pub fn from_file_request(
        user_id: Uuid,
        request: &FileUploadRequest,
        body: Bytes,
    ) -> Result<Self, ForumError> {
        if body.len() as u64 != request.size_bytes {
            return Err(ForumError::Validation("上传内容大小不一致".to_string()));
        }
        let filename = safe_filename(&request.original_filename)?;
        let bucket = bucket_for_purpose(&request.purpose).to_string();
        Ok(Self {
            bucket,
            object_key: format!("{user_id}/{}/{filename}", request.sha256),
            content_type: request.mime_type.clone(),
            body,
        })
    }

    pub fn public_url(&self) -> String {
        format!("/files/{}/{}", self.bucket, self.object_key)
    }
}

#[cfg(feature = "ssr")]
pub struct RustFsObjectStore {
    client: aws_sdk_s3::Client,
}

#[cfg(feature = "ssr")]
impl RustFsObjectStore {
    pub async fn from_env(endpoint_url: &str) -> Self {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(endpoint_url)
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build();
        Self {
            client: aws_sdk_s3::Client::from_conf(s3_config),
        }
    }

    pub async fn put_object(&self, upload: ObjectUpload) -> Result<(), ForumError> {
        self.client
            .put_object()
            .bucket(upload.bucket)
            .key(upload.object_key)
            .content_type(upload.content_type)
            .body(aws_sdk_s3::primitives::ByteStream::from(upload.body))
            .send()
            .await
            .map_err(|_| ForumError::Internal)?;
        Ok(())
    }
}

pub fn safe_filename(filename: &str) -> Result<String, ForumError> {
    let filename = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        .collect::<String>();
    if filename.is_empty() {
        Err(ForumError::Validation("文件名不能为空".to_string()))
    } else {
        Ok(filename)
    }
}

pub fn bucket_for_purpose(purpose: &FilePurpose) -> &'static str {
    match purpose {
        FilePurpose::Avatar => "avatars",
        FilePurpose::PostCover | FilePurpose::PostImage => "post-images",
        FilePurpose::AnnouncementImage => "announcement-images",
    }
}
