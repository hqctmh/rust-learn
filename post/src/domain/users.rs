use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::posts::PostSummary;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserSpace {
    pub profile: UserProfile,
    pub stats: UserStats,
    pub is_me: bool,
    pub followed_by_viewer: bool,
    pub published_posts: Vec<PostSummary>,
    pub draft_posts: Vec<PostSummary>,
    pub comments: Vec<UserCommentItem>,
    pub favorite_posts: Vec<PostSummary>,
    pub following: Vec<UserProfile>,
    pub followers: Vec<UserProfile>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub username: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub bio: String,
    pub registered_at: OffsetDateTime,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserStats {
    pub following: usize,
    pub followers: usize,
    pub published_posts: usize,
    pub received_likes: i64,
    pub received_favorites: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserCommentItem {
    pub post_id: Uuid,
    pub post_title: String,
    pub content: String,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateProfileRequest {
    pub nickname: String,
    pub bio: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateAvatarRequest {
    pub avatar_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}
