use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct IdleDeadline {
    timeout: Duration,
    deadline: Instant,
}

impl IdleDeadline {
    pub fn new(now: Instant, timeout: Duration) -> Self {
        Self {
            timeout,
            deadline: now + timeout,
        }
    }

    pub fn reset(&mut self, now: Instant) {
        self.deadline = now + self.timeout;
    }

    pub fn remaining(&self, now: Instant) -> Option<Duration> {
        self.deadline.checked_duration_since(now)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::IdleDeadline;

    #[test]
    fn idle_deadline_only_moves_after_business_activity() {
        let start = Instant::now();
        let mut deadline = IdleDeadline::new(start, Duration::from_secs(60));

        assert_eq!(
            deadline.remaining(start + Duration::from_secs(15)),
            Some(Duration::from_secs(45))
        );
        deadline.reset(start + Duration::from_secs(20));
        assert_eq!(
            deadline.remaining(start + Duration::from_secs(50)),
            Some(Duration::from_secs(30))
        );
        assert_eq!(deadline.remaining(start + Duration::from_secs(81)), None);
    }
}
