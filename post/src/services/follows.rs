use std::collections::HashSet;

use uuid::Uuid;

use crate::{domain::reactions::FollowState, error::ForumError};

pub struct FollowService;

impl FollowService {
    pub fn toggle_follow(
        follows: &mut HashSet<(Uuid, Uuid)>,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> Result<FollowState, ForumError> {
        if follower_id == followee_id {
            return Err(ForumError::Conflict("不能关注自己".to_string()));
        }

        let key = (follower_id, followee_id);
        let following = if follows.contains(&key) {
            follows.remove(&key);
            false
        } else {
            follows.insert(key);
            true
        };

        Ok(FollowState {
            follower_id,
            followee_id,
            following,
        })
    }
}
