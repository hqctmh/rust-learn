use std::collections::HashSet;

use uuid::Uuid;

pub struct ReactionService;

impl ReactionService {
    pub fn toggle_pair(set: &mut HashSet<(Uuid, Uuid)>, key: (Uuid, Uuid)) -> bool {
        if set.contains(&key) {
            set.remove(&key);
            false
        } else {
            set.insert(key);
            true
        }
    }

    pub fn apply_counter_delta(count: &mut i64, active: bool) -> i64 {
        if active {
            *count += 1;
        } else {
            *count = (*count - 1).max(0);
        }
        *count
    }
}
