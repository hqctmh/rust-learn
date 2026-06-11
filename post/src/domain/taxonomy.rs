use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CategoryItem {
    pub category_id: Uuid,
    pub name: String,
    pub color: String,
    pub sort_order: i32,
    pub enabled: bool,
    pub post_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TagItem {
    pub tag_id: Uuid,
    pub name: String,
    pub sort_order: i32,
    pub enabled: bool,
    pub use_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub color: String,
    pub sort_order: i32,
}

impl CreateCategoryRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_name(&self.name, "分类名称")?;
        validate_color(&self.color)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub enabled: Option<bool>,
}

impl UpdateCategoryRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(name) = &self.name {
            validate_name(name, "分类名称")?;
        }
        if let Some(color) = &self.color {
            validate_color(color)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateTagRequest {
    pub name: String,
    pub sort_order: i32,
}

impl CreateTagRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_name(&self.name, "标签名称")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
    pub enabled: Option<bool>,
    pub use_count: Option<u32>,
}

impl UpdateTagRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(name) = &self.name {
            validate_name(name, "标签名称")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MergeTagRequest {
    pub target_tag_id: Uuid,
}

fn validate_name(value: &str, label: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > 32 {
        return Err(format!("{label}不能超过 32 个字符"));
    }
    Ok(())
}

fn validate_color(value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.len() != 7
        || !value.starts_with('#')
        || !value[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err("分类颜色必须是 #RRGGBB".to_string());
    }
    Ok(())
}
