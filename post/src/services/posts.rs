use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        auth::SessionUser,
        posts::{
            AutosaveDraftRequest, CreatePostRequest, PostDetail, PostStatus, PostSummary,
            UpdatePostRequest,
        },
    },
    error::ForumError,
};

pub struct PostAuthoringService;

impl PostAuthoringService {
    pub fn preview_markdown(markdown: &str) -> Result<String, ForumError> {
        let markdown = markdown.trim();
        if markdown.is_empty() {
            return Err(ForumError::Validation("正文不能为空".to_string()));
        }

        Ok(render_markdown_safe(markdown))
    }

    pub fn build_post(
        post_id: Uuid,
        author: &SessionUser,
        request: CreatePostRequest,
        now: OffsetDateTime,
    ) -> Result<PostDetail, ForumError> {
        let publish = request.publish;
        let (title, markdown) = validate_editor_input(&request.title, &request.markdown)?;
        let published_at = publish.then_some(now);

        Ok(PostDetail {
            summary: PostSummary {
                post_id,
                title: title.clone(),
                summary: normalize_summary(&request.summary, &markdown),
                author_id: author.user_id,
                author_name: author.nickname.clone(),
                author_avatar_url: author.avatar_url.clone(),
                category_name: normalize_category(request.category_name),
                tags: normalize_tags(request.tag_names),
                view_count: 0,
                comment_count: 0,
                like_count: 0,
                favorite_count: 0,
                published_at,
            },
            markdown: markdown.clone(),
            sanitized_html: render_markdown_safe(&markdown),
            status: if publish {
                PostStatus::Published
            } else {
                PostStatus::Draft
            },
            liked_by_me: false,
            favorited_by_me: false,
            following_author: false,
        })
    }

    pub fn build_draft(
        post_id: Uuid,
        author: &SessionUser,
        request: AutosaveDraftRequest,
    ) -> Result<PostDetail, ForumError> {
        let (title, markdown) = validate_editor_input(&request.title, &request.markdown)?;

        Ok(PostDetail {
            summary: PostSummary {
                post_id,
                title: title.clone(),
                summary: normalize_summary(&request.summary, &markdown),
                author_id: author.user_id,
                author_name: author.nickname.clone(),
                author_avatar_url: author.avatar_url.clone(),
                category_name: normalize_category(request.category_name),
                tags: normalize_tags(request.tag_names),
                view_count: 0,
                comment_count: 0,
                like_count: 0,
                favorite_count: 0,
                published_at: None,
            },
            markdown: markdown.clone(),
            sanitized_html: render_markdown_safe(&markdown),
            status: PostStatus::Draft,
            liked_by_me: false,
            favorited_by_me: false,
            following_author: false,
        })
    }

    pub fn apply_autosave(
        detail: &mut PostDetail,
        request: AutosaveDraftRequest,
    ) -> Result<(), ForumError> {
        let (title, markdown) = validate_editor_input(&request.title, &request.markdown)?;
        detail.summary.title = title;
        detail.summary.summary = normalize_summary(&request.summary, &markdown);
        detail.summary.category_name = normalize_category(request.category_name);
        detail.summary.tags = normalize_tags(request.tag_names);
        detail.markdown = markdown.clone();
        detail.sanitized_html = render_markdown_safe(&markdown);
        Ok(())
    }

    pub fn apply_update(
        detail: &mut PostDetail,
        request: UpdatePostRequest,
        now: OffsetDateTime,
    ) -> Result<(), ForumError> {
        let publish = request.publish;
        let (title, markdown) = validate_editor_input(&request.title, &request.markdown)?;
        detail.summary.title = title;
        detail.summary.summary = normalize_summary(&request.summary, &markdown);
        detail.summary.category_name = normalize_category(request.category_name);
        detail.summary.tags = normalize_tags(request.tag_names);
        if publish && detail.summary.published_at.is_none() {
            detail.summary.published_at = Some(now);
        }
        if !publish {
            detail.summary.published_at = None;
        }
        detail.status = if publish {
            PostStatus::Published
        } else {
            PostStatus::Draft
        };
        detail.markdown = markdown.clone();
        detail.sanitized_html = render_markdown_safe(&markdown);
        Ok(())
    }
}

fn validate_editor_input(title: &str, markdown: &str) -> Result<(String, String), ForumError> {
    let title = title.trim();
    let markdown = markdown.trim();

    if title.is_empty() {
        return Err(ForumError::Validation("标题不能为空".to_string()));
    }
    if markdown.is_empty() {
        return Err(ForumError::Validation("正文不能为空".to_string()));
    }

    Ok((title.to_string(), markdown.to_string()))
}

fn normalize_category(category_name: Option<String>) -> Option<String> {
    category_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_summary(summary: &str, markdown: &str) -> String {
    let summary = summary.trim();
    if !summary.is_empty() {
        return summary.chars().take(180).collect();
    }

    markdown
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("暂未填写摘要")
        .trim_start_matches('#')
        .trim()
        .chars()
        .take(180)
        .collect()
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|tag| tag.trim().trim_start_matches('#').to_lowercase())
        .filter(|tag| !tag.is_empty())
        .fold(Vec::new(), |mut acc, tag| {
            if !acc.contains(&tag) {
                acc.push(tag);
            }
            acc
        })
}

fn render_markdown_safe(markdown: &str) -> String {
    markdown
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            let escaped = escape_html(line);
            if let Some(heading) = escaped.strip_prefix("# ") {
                Some(format!("<h1>{heading}</h1>"))
            } else if let Some(heading) = escaped.strip_prefix("## ") {
                Some(format!("<h2>{heading}</h2>"))
            } else if let Some(quote) = escaped.strip_prefix("&gt; ") {
                Some(format!("<blockquote>{quote}</blockquote>"))
            } else {
                Some(format!("<p>{escaped}</p>"))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
