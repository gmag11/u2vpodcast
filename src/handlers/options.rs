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

pub fn api_options(cfg: &mut web::ServiceConfig) {
    cfg.service(update).service(post_options);
}

#[get("/options/update/")]
async fn update(data: Data<AppState>, session: Session) -> impl Responder {
    info!("update");
    // Non-blocking: run the full refresh in the background and return
    // immediately. Completion is observable through per-channel sync status
    // (last_sync_at/last_sync_ok/last_sync_error).
    if SYNC_IN_PROGRESS.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err(Error::new_with_status_code(
            "A full sync is already running",
            StatusCode::CONFLICT,
        ));
    }
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
