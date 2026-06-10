use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ReactionTarget {
    Post(Uuid),
    Comment(Uuid),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToggleResult {
    pub active: bool,
    pub count: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FollowState {
    pub follower_id: Uuid,
    pub followee_id: Uuid,
    pub following: bool,
}
