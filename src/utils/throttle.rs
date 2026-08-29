//! Single-connection YouTube throttle with a post-connection cooldown
//! (youtube-throttling / limit-youtube-concurrency).
//!
//! A process-wide semaphore of size 1 serializes every YouTube-bound
//! operation: metadata fetches, cover image probes/downloads, and every
//! yt-dlp execution (listing, download, periodic update). Each holder keeps
//! the slot while its connection runs AND for the cooldown afterwards, so
//! consecutive connections are at least `cooldown` apart — even when the
//! previous holder failed.

use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Default pause between consecutive YouTube connections when the
/// `cooldown_seconds` config key is absent.
pub const DEFAULT_COOLDOWN: Duration = Duration::from_secs(3);

/// The single slot shared by every call site. Initialized lazily (also from
/// [`init_throttle`] at startup) so the module never requires wiring through
/// `AppState` into the model/worker layers.
static YT_SLOT: OnceLock<Arc<Semaphore>> = OnceLock::new();

/// The configured post-connection cooldown; falls back to
/// [`DEFAULT_COOLDOWN`] when never initialized.
static YT_COOLDOWN: OnceLock<Duration> = OnceLock::new();

/// Configure the throttle at startup (task 1.2). Idempotent: the first call
/// wins, so tests and embedded callers cannot re-clobber the runtime value.
pub fn init_throttle(cooldown: Duration) {
    let _ = YT_COOLDOWN.set(cooldown);
    let _ = YT_SLOT.set(Arc::new(Semaphore::new(1)));
}

fn global_slot() -> Arc<Semaphore> {
    YT_SLOT.get_or_init(|| Arc::new(Semaphore::new(1))).clone()
}

pub fn cooldown_duration() -> Duration {
    *YT_COOLDOWN.get().unwrap_or(&DEFAULT_COOLDOWN)
}

/// RAII guard holding a single-connection slot.
///
/// Normal path: keep it alive across the connection work, then call
/// [`YoutubeGuard::cooldown_and_release`] so the slot stays held through the
/// post-connection cooldown (on both success and error). If the guard is
/// dropped without that call (panic, early return), the slot is released
/// immediately — waiters never deadlock (youtube-throttling "no deadlock").
#[must_use = "dropping the guard without cooldown_and_release skips the cooldown"]
pub struct YoutubeGuard {
    permit: Option<OwnedSemaphorePermit>,
    cooldown: Duration,
}

impl YoutubeGuard {
    /// Acquire the process-wide slot, waiting for the current holder to finish
    /// its work and cooldown. Bounded: the held slot resumes within
    /// (work + cooldown) time.
    pub async fn acquire() -> Self {
        Self::acquire_on(global_slot(), cooldown_duration()).await
    }

    /// Acquire a specific slot (used by tests to isolate timings from the
    /// process-wide slot and by [`YoutubeGuard::acquire`] for the global one).
    async fn acquire_on(slot: Arc<Semaphore>, cooldown: Duration) -> Self {
        let permit = slot
            .acquire_owned()
            .await
            .expect("YouTube throttle semaphore is never closed");
        Self {
            permit: Some(permit),
            cooldown,
        }
    }

    /// Sleep the cooldown while still holding the slot, then release it. Use
    /// on the success AND failure paths so a burst of errors cannot turn into
    /// rapid-fire retries.
    pub async fn cooldown_and_release(mut self) {
        tokio::time::sleep(self.cooldown).await;
        self.permit.take();
    }
}

impl Drop for YoutubeGuard {
    fn drop(&mut self) {
        // Panic / early-return safety: release the slot immediately. After
        // `cooldown_and_release` the permit is already gone; dropping again is
        // a no-op (no double release).
        self.permit.take();
    }
}

/// Run `work` under the process-wide YouTube throttle: acquire the slot, run
/// `work`, then enforce the cooldown (success or error) and release. `work` is
/// a closure returning a future so the slot is acquired before the connection
/// inside it starts. Returns whatever the future produced.
pub async fn with_youtube_slot<T, F, Fut>(work: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let guard = YoutubeGuard::acquire().await;
    let result = work().await;
    guard.cooldown_and_release().await;
    result
}

#[cfg(test)]
mod throttle_tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::sync::Barrier;

    #[tokio::test]
    async fn concurrent_work_never_overlaps() {
        let slot = Arc::new(Semaphore::new(1));
        let active = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(4));
        let mut handles = Vec::new();
        for _ in 0..4 {
            let slot = Arc::clone(&slot);
            let active = Arc::clone(&active);
            let barrier = Arc::clone(&barrier);
            handles.push(tokio::spawn(async move {
                // All four tasks line up at the barrier so they attempt to
                // acquire the slot simultaneously.
                barrier.wait().await;
                let guard = YoutubeGuard::acquire_on(slot, Duration::from_millis(5)).await;
                let previous = active.fetch_add(1, Ordering::SeqCst);
                assert_eq!(previous, 0, "two ops overlapped in the slot");
                tokio::time::sleep(Duration::from_millis(40)).await;
                active.fetch_sub(1, Ordering::SeqCst);
                guard.cooldown_and_release().await;
                7usize
            }));
        }
        for handle in handles {
            assert_eq!(handle.await.expect("task did not panic"), 7);
        }
    }

    #[tokio::test]
    async fn cooldown_holds_the_slot_after_success() {
        let slot = Arc::new(Semaphore::new(1));
        let started = std::time::Instant::now();
        let guard = YoutubeGuard::acquire_on(slot, Duration::from_millis(30)).await;
        // work succeeds
        guard.cooldown_and_release().await;
        assert!(
            started.elapsed() >= Duration::from_millis(20),
            "cooldown must close the gap after a successful connection"
        );
    }

    #[tokio::test]
    async fn cooldown_holds_the_slot_after_error() {
        let slot = Arc::new(Semaphore::new(1));
        let started = std::time::Instant::now();
        // A failing connection still holds the slot through the cooldown.
        let guard = YoutubeGuard::acquire_on(slot, Duration::from_millis(30)).await;
        let result: Result<usize, &str> = Err("boom"); // the connection failed
        let _ = result; // keep the failed outcome flowing through the caller
        guard.cooldown_and_release().await;
        assert!(
            started.elapsed() >= Duration::from_millis(20),
            "a failed connection must still enforce the cooldown"
        );
    }

    #[tokio::test]
    async fn panic_releases_the_slot_immediately() {
        let slot = Arc::new(Semaphore::new(1));
        let handle = tokio::spawn({
            let slot = Arc::clone(&slot);
            async move {
                let _guard = YoutubeGuard::acquire_on(slot, Duration::from_secs(60)).await;
                panic!("intentional throttle panic");
            }
        });
        assert!(handle.await.is_err(), "the holder task must have panicked");

        // The slot must be free right away: the panicking holder dropped the
        // guard without holding the (60s) cooldown.
        let started = std::time::Instant::now();
        let guard = YoutubeGuard::acquire_on(Arc::clone(&slot), Duration::from_secs(60)).await;
        assert!(
            started.elapsed() < Duration::from_millis(100),
            "a panicking holder must not hold the slot"
        );
        drop(guard); // immediate release, no cooldown sleep
    }

    #[tokio::test]
    async fn second_waiter_starts_only_after_the_cooldown() {
        let slot = Arc::new(Semaphore::new(1));
        let guard = YoutubeGuard::acquire_on(slot.clone(), Duration::from_millis(30)).await;
        let slot_free = Arc::new(AtomicUsize::new(0));
        let flag = Arc::clone(&slot_free);
        let task = tokio::spawn(async move {
            let g = YoutubeGuard::acquire_on(slot, Duration::from_millis(1)).await;
            flag.store(1, Ordering::SeqCst);
            drop(g);
        });
        // Give the task a moment to start waiting: it must NOT be in the slot
        // while the guard is held.
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert_eq!(
            slot_free.load(Ordering::SeqCst),
            0,
            "waiter must wait for the holder"
        );
        guard.cooldown_and_release().await;
        task.await.expect("waiter task completed");
        assert_eq!(
            slot_free.load(Ordering::SeqCst),
            1,
            "waiter resumed after release"
        );
    }

    #[tokio::test]
    async fn independent_async_work_progresses_while_slot_held() {
        let slot = Arc::new(Semaphore::new(1));
        // While a YouTube operation holds the slot, unrelated async work must
        // keep running (the throttle must never stall login/status/other
        // endpoints — task 3.2).
        let guard = YoutubeGuard::acquire_on(slot, Duration::from_millis(1000)).await;
        let started = std::time::Instant::now();
        let independent = tokio::spawn(async {
            tokio::time::sleep(Duration::from_millis(40)).await;
            "done"
        });
        let result = independent.await.expect("independent task completed");
        assert_eq!(result, "done");
        assert!(
            started.elapsed() < Duration::from_millis(500),
            "unrelated async work must not be blocked by the held YouTube slot"
        );
        drop(guard); // immediate release, no cooldown sleep
    }
}
