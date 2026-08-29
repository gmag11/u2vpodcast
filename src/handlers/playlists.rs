use actix_session::Session;
use actix_web::{
    delete,
    get,
    post,
    put,
    web::{
        Data,
        Json,
        Path,
        ServiceConfig,
    },
    HttpResponse,
};
use serde::Deserialize;
use tracing::{
    error,
    info,
};

use super::{
    AppState,
    super::models::{
        CResponse,
        Episode,
        PlaylistItem,
    },
};

pub fn api_playlists(cfg: &mut ServiceConfig) {
    cfg.service(read)
        .service(add)
        .service(remove)
        .service(reorder);
}

#[get("/playlist/")]
async fn read(
    data: Data<AppState>,
    session: Session,
) -> HttpResponse {
    info!("read playlist");
    match PlaylistItem::read_episodes_with_channels(&data.pool).await {
        Ok(mut episodes) => {
            for episode in &mut episodes {
                episode.apply_sponsorblock_config(
                    data.config.sponsorblock_enabled,
                    &data.config.sponsorblock_rejected_categories,
                );
            }
            CResponse::ok(session, episodes)
        },
        Err(e) => {
            error!("Error reading playlist: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

#[derive(Deserialize)]
struct AddBody {
    episode_id: i64,
}

#[post("/playlist/")]
async fn add(
    data: Data<AppState>,
    session: Session,
    body: Json<AddBody>,
) -> HttpResponse {
    info!("add to playlist");
    let episode_id = body.into_inner().episode_id;
    // Referential integrity is handler-enforced (no FK constraint): an unknown
    // episode is a 404, not a masked database error.
    if let Err(e) = Episode::read(&data.pool, episode_id).await {
        error!("Error: {e}");
        return CResponse::ko(e.status_code(), session);
    }
    match PlaylistItem::add(&data.pool, episode_id).await {
        Ok(item) => CResponse::ok(session, item),
        Err(e) => {
            error!("Error adding to playlist: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

#[derive(Deserialize)]
struct RemovePath {
    episode_id: i64,
}

#[delete("/playlist/{episode_id}/")]
async fn remove(
    data: Data<AppState>,
    session: Session,
    path: Path<RemovePath>,
) -> HttpResponse {
    info!("remove from playlist");
    match PlaylistItem::remove(&data.pool, path.into_inner().episode_id).await {
        Ok(item) => CResponse::ok(session, item),
        Err(e) => {
            error!("Error removing from playlist: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

#[derive(Deserialize)]
struct ReorderBody {
    episode_ids: Vec<i64>,
}

#[put("/playlist/reorder/")]
async fn reorder(
    data: Data<AppState>,
    session: Session,
    body: Json<ReorderBody>,
) -> HttpResponse {
    info!("reorder playlist");
    match PlaylistItem::reorder(&data.pool, &body.into_inner().episode_ids).await {
        Ok(()) => CResponse::ok(session, serde_json::json!(true)),
        Err(e) => {
            error!("Error reordering playlist: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}