use actix_session::Session;
use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{self, Data, Json},
    Responder,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use super::{
    super::{
        models::{CResponse, Error, Param},
        utils::worker::do_the_work,
    },
    AppState,
};

#[derive(Serialize, Deserialize)]
struct KeyValue {
    key: String,
    value: String,
}

// Guards against stacking manual full syncs (non-blocking-update-paths): the
// scheduled worker loop is exempt and manages its own cadence.
static SYNC_IN_PROGRESS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// Minimum interval between manual full syncs (DoS protection). Every manual
// sync triggers a full re-scan with yt-dlp/ffmpeg work, so it must not be
// spammable. The worker loop is unaffected.
const MANUAL_SYNC_COOLDOWN_SECS: u64 = 300;
static LAST_MANUAL_SYNC: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// True when `now` is inside the cooldown window opened by the sync at `last`.
fn within_cooldown(now: u64, last: u64, cooldown_secs: u64) -> bool {
    now.saturating_sub(last) < cooldown_secs
}

pub fn api_options(cfg: &mut web::ServiceConfig) {
    cfg.service(update).service(post_options);
}

#[get("/options/update/")]
async fn update(data: Data<AppState>, session: Session) -> impl Responder {
    info!("update");
    // Rate limit: reject manual syncs started within the cooldown window.
    let now = unix_secs();
    let last = LAST_MANUAL_SYNC.load(std::sync::atomic::Ordering::SeqCst);
    if within_cooldown(now, last, MANUAL_SYNC_COOLDOWN_SECS) {
        return Err(Error::new_with_status_code(
            "A manual sync was triggered recently; wait before starting another",
            StatusCode::TOO_MANY_REQUESTS,
        ));
    }
    // Non-blocking: run the full refresh in the background and return
    // immediately. Completion is observable through per-channel sync status
    // (last_sync_at/last_sync_ok/last_sync_error).
    if SYNC_IN_PROGRESS
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        return Err(Error::new_with_status_code(
            "A full sync is already running",
            StatusCode::CONFLICT,
        ));
    }
    LAST_MANUAL_SYNC.store(now, std::sync::atomic::Ordering::SeqCst);
    let pool = data.pool.clone();
    let config = data.config.clone();
    actix_web::rt::spawn(async move {
        let result = do_the_work(&pool, &config).await;
        SYNC_IN_PROGRESS.store(false, std::sync::atomic::Ordering::SeqCst);
        match result {
            Ok(()) => info!("Manual full sync finished"),
            Err(e) => error!("Manual full sync failed: {e}"),
        }
    });
    Ok(CResponse::ok(session, ""))
}

#[post("/options/")]
async fn post_options(
    data: Data<AppState>,
    session: Session,
    pairs: Json<Vec<KeyValue>>,
) -> impl Responder {
    info!("post_options");
    let mut response_pairs = Vec::new();
    for pair in pairs.into_inner().as_slice() {
        match Param::set(&data.pool, &pair.key, &pair.value).await {
            Ok(kv) => {
                debug!("{:?}", kv);
                let key = kv.get_key();
                let value = kv.get_value();
                response_pairs.push(KeyValue {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }
            Err(e) => {
                error!("{:?}", e);
                response_pairs.push(KeyValue {
                    key: pair.key.clone(),
                    value: pair.value.clone(),
                });
            }
        }
    }
    CResponse::ok(session, response_pairs)
}

#[cfg(test)]
mod cooldown_tests {
    use super::within_cooldown;

    const COOLDOWN: u64 = 300;

    #[test]
    fn rejects_syncs_inside_the_cooldown_window() {
        assert!(within_cooldown(0, 0, COOLDOWN), "same-second sync is rejected");
        assert!(within_cooldown(100, 50, COOLDOWN));
        assert!(within_cooldown(349, 50, COOLDOWN));
    }

    #[test]
    fn allows_syncs_after_the_cooldown_elapses() {
        assert!(!within_cooldown(350, 50, COOLDOWN));
        assert!(!within_cooldown(3600, 50, COOLDOWN));
    }

    #[test]
    fn never_started_last_sync_is_allowed() {
        // LAST_MANUAL_SYNC defaults to 0; a sync in the far past (or the unix
        // epoch) must not be treated as an active cooldown.
        assert!(!within_cooldown(1_700_000_000, 0, COOLDOWN));
    }
}
