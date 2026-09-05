use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// Failed-login window and cap per (IP, username) before a login attempt is
// rejected with 429. After the window slides past the cap, the limiter allows
// new attempts again (sliding window, no permanent lockout).
const FAILURE_WINDOW: Duration = Duration::from_secs(15 * 60);
const MAX_FAILURES: usize = 5;

// In-memory per-key sliding-window of failed login timestamps. Single-process
// app (actix runs one process here); a shared instance is enough.
pub struct LoginRateLimiter {
    failures: Mutex<HashMap<String, Vec<Instant>>>,
}

impl LoginRateLimiter {
    pub fn new() -> Self {
        Self {
            failures: Mutex::new(HashMap::new()),
        }
    }

    fn prune(timestamps: &mut Vec<Instant>) {
        let cutoff = Instant::now() - FAILURE_WINDOW;
        timestamps.retain(|t| *t > cutoff);
    }

    pub fn is_blocked(&self, key: &str) -> bool {
        let mut failures = self.failures.lock().unwrap();
        let timestamps = failures.entry(key.to_string()).or_default();
        Self::prune(timestamps);
        timestamps.len() >= MAX_FAILURES
    }

    pub fn record_failure(&self, key: &str) {
        let mut failures = self.failures.lock().unwrap();
        let timestamps = failures.entry(key.to_string()).or_default();
        Self::prune(timestamps);
        timestamps.push(Instant::now());
    }

    pub fn record_success(&self, key: &str) {
        let mut failures = self.failures.lock().unwrap();
        failures.remove(key);
    }
}

impl Default for LoginRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn blocks_after_max_failures_within_window() {
        let limiter = LoginRateLimiter::new();
        for _ in 0..MAX_FAILURES {
            assert!(!limiter.is_blocked("1.2.3.4|admin"));
            limiter.record_failure("1.2.3.4|admin");
        }
        assert!(limiter.is_blocked("1.2.3.4|admin"));
        // A different key is unaffected.
        assert!(!limiter.is_blocked("9.9.9.9|admin"));
    }

    #[test]
    fn success_clears_failures() {
        let limiter = LoginRateLimiter::new();
        for _ in 0..MAX_FAILURES {
            limiter.record_failure("1.2.3.4|admin");
        }
        assert!(limiter.is_blocked("1.2.3.4|admin"));
        limiter.record_success("1.2.3.4|admin");
        assert!(!limiter.is_blocked("1.2.3.4|admin"));
    }

    #[test]
    fn old_failures_expire_and_unblock() {
        let limiter = LoginRateLimiter::new();
        for _ in 0..MAX_FAILURES {
            limiter.record_failure("1.2.3.4|admin");
        }
        assert!(limiter.is_blocked("1.2.3.4|admin"));
        // Slide past the window by rewriting timestamps directly (test-only
        // access to the internals) and confirm the cap resets.
        {
            let mut failures = limiter.failures.lock().unwrap();
            let now = Instant::now();
            let old = now - FAILURE_WINDOW - Duration::from_secs(1);
            let expired = failures.get_mut("1.2.3.4|admin").unwrap();
            for t in expired.iter_mut() {
                *t = old;
            }
        }
        thread::sleep(Duration::from_millis(5));
        assert!(!limiter.is_blocked("1.2.3.4|admin"));
    }
}