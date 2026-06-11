use uuid::Uuid;

use crate::{
    domain::taxonomy::{
        CategoryItem, CreateCategoryRequest, CreateTagRequest, TagItem, UpdateCategoryRequest,
        UpdateTagRequest,
    },
    error::ForumError,
};

pub struct TaxonomyService;

impl TaxonomyService {
    pub fn build_category(
        category_id: Uuid,
        request: CreateCategoryRequest,
    ) -> Result<CategoryItem, ForumError> {
        request.validate().map_err(ForumError::Validation)?;

        Ok(CategoryItem {
            category_id,
            name: request.name.trim().to_string(),
            color: request.color.trim().to_string(),
            sort_order: request.sort_order,
            enabled: true,
            post_count: 0,
        })
    }

    pub fn apply_category_update(
        category: &mut CategoryItem,
        request: UpdateCategoryRequest,
    ) -> Result<(), ForumError> {
        request.validate().map_err(ForumError::Validation)?;

        if let Some(name) = request.name {
            category.name = name.trim().to_string();
        }
        if let Some(color) = request.color {
            category.color = color.trim().to_string();
        }
        if let Some(sort_order) = request.sort_order {
            category.sort_order = sort_order;
        }
        if let Some(enabled) = request.enabled {
            category.enabled = enabled;
        }

        Ok(())
    }

    pub fn build_tag(tag_id: Uuid, request: CreateTagRequest) -> Result<TagItem, ForumError> {
        request.validate().map_err(ForumError::Validation)?;

        Ok(TagItem {
            tag_id,
            name: normalize_tag_name(request.name),
            sort_order: request.sort_order,
            enabled: true,
            use_count: 0,
        })
    }

    pub fn apply_tag_update(
        tag: &mut TagItem,
        request: UpdateTagRequest,
    ) -> Result<(), ForumError> {
        request.validate().map_err(ForumError::Validation)?;

        if let Some(name) = request.name {
            tag.name = normalize_tag_name(name);
        }
        if let Some(sort_order) = request.sort_order {
            tag.sort_order = sort_order;
        }
        if let Some(enabled) = request.enabled {
            tag.enabled = enabled;
        }
        if let Some(use_count) = request.use_count {
            tag.use_count = use_count;
        }

        Ok(())
    }

    pub fn validate_tag_merge(source_tag_id: Uuid, target_tag_id: Uuid) -> Result<(), ForumError> {
        if source_tag_id == target_tag_id {
            return Err(ForumError::Conflict("不能合并到同一个标签".to_string()));
        }

        Ok(())
    }

    pub fn apply_target_merge(target: &mut TagItem, source_use_count: u32) {
        target.use_count += source_use_count;
        target.enabled = true;
    }

    pub fn disable_merged_source(source: &mut TagItem) {
        source.enabled = false;
        source.use_count = 0;
    }
}

fn normalize_tag_name(name: String) -> String {
    name.trim().to_lowercase()
}
