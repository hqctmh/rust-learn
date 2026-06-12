use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::files::{FileAsset, FileUsage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileAssetRow {
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

impl From<FileAssetRow> for FileAsset {
    fn from(row: FileAssetRow) -> Self {
        Self {
            file_id: row.file_id,
            original_filename: row.original_filename,
            bucket: row.bucket,
            storage_key: row.storage_key,
            file_size: row.file_size,
            mime_type: row.mime_type,
            file_hash: row.file_hash,
            uploader_id: row.uploader_id,
            public_url: row.public_url,
            markdown_image: row.markdown_image,
            uploaded_at: row.uploaded_at,
        }
    }
}

pub struct PostgresFileRepository;

impl PostgresFileRepository {
    pub async fn find_by_hash(
        pool: &sqlx::PgPool,
        file_hash: &str,
    ) -> sqlx::Result<Option<FileAsset>> {
        let row = sqlx::query_as!(
            FileAssetRow,
            r#"
select
    file_id,
    original_filename,
    bucket,
    storage_key,
    file_size,
    mime_type,
    file_hash,
    uploader_id,
    public_url,
    markdown_image,
    uploaded_at
from file_assets
where file_hash = $1
limit 1
"#,
            file_hash
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(FileAsset::from))
    }

    pub async fn insert_asset(
        pool: &sqlx::PgPool,
        asset: &FileAsset,
        usage: &FileUsage,
    ) -> sqlx::Result<FileAsset> {
        let usage = usage_as_str(usage);
        let row = sqlx::query_as!(
            FileAssetRow,
            r#"
insert into file_assets (
    file_id,
    original_filename,
    bucket,
    storage_key,
    file_size,
    mime_type,
    file_hash,
    usage,
    uploader_id,
    public_url,
    markdown_image,
    uploaded_at
)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
returning
    file_id,
    original_filename,
    bucket,
    storage_key,
    file_size,
    mime_type,
    file_hash,
    uploader_id,
    public_url,
    markdown_image,
    uploaded_at
"#,
            asset.file_id,
            asset.original_filename,
            asset.bucket,
            asset.storage_key,
            asset.file_size,
            asset.mime_type,
            asset.file_hash,
            usage,
            asset.uploader_id,
            asset.public_url,
            asset.markdown_image,
            asset.uploaded_at
        )
        .fetch_one(pool)
        .await?;

        Ok(FileAsset::from(row))
    }
}

fn usage_as_str(usage: &FileUsage) -> &'static str {
    match usage {
        FileUsage::Avatar => "avatar",
        FileUsage::CoverImage => "cover_image",
        FileUsage::MarkdownImage => "markdown_image",
        FileUsage::AnnouncementImage => "announcement_image",
    }
}
