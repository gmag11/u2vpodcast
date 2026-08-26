use actix_web::{
    Responder,
    HttpResponse,
    get,
    put,
    web::{
        Path,
        Data,
        Json,
    },
};
use actix_session::Session;
use serde::Deserialize;
use tracing::{
    info,
    error,
    debug,
};

use super::{
    AppState,
    super::models::{
        Channel,
        Episode,
        EpisodeProgress,
        CResponse,
    },
};

#[get("/channels/{channel}/episodes/")]
async fn read_with_pagination(
    data: Data<AppState>,
    session: Session,
    path: Path<String>
) -> impl Responder{
    info!("read_api_channels");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await{
        Ok(channel) => match Episode::read_episodes_for_channel(&data.pool, channel.id).await{
            Ok(mut episodes) => {
                debug!("{:?}", episodes);
                for episode in episodes.iter_mut(){
                    episode.channel_slug = channel.slug.clone();
                }
                Ok(CResponse::ok(session, episodes))
            },
            Err(e) => {
                error!("{e}");
                Err(e)
            }
        },
        Err(e) => {
            error!("{e}");
            Err(e)
        }
    }
}

#[get("/episodes/")]
async fn read_all(
    data: Data<AppState>,
    session: Session,
) -> impl Responder{
    info!("read_all");
    match Episode::read_all_with_channels(&data.pool).await{
        Ok(episodes) => Ok(CResponse::ok(session, episodes)),
        Err(e) => {
            error!("{e}");
            Err(e)
        }
    }
}

#[derive(Deserialize)]
pub struct ProgressBody {
    pub position_seconds: i64,
    pub listened: bool,
}

#[get("/episodes/{yt_id}/progress/")]
async fn read_progress(
    data: Data<AppState>,
    session: Session,
    yt_id: Path<String>,
) -> actix_web::HttpResponse {
    info!("read_progress");
    match Episode::read_progress_by_yt_id(&data.pool, &yt_id.into_inner()).await{
        Ok(progress) => {
            let progress: EpisodeProgress = progress;
            CResponse::ok(session, progress)
        },
        Err(e) => {
            // A missing episode is 404; any other failure (e.g. a real
            // database error) surfaces its own status instead of being masked.
            error!("Error reading progress: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

#[put("/episodes/{yt_id}/progress/")]
async fn update_progress(
    data: Data<AppState>,
    session: Session,
    yt_id: Path<String>,
    body: Json<ProgressBody>,
) -> actix_web::HttpResponse {
    info!("update_progress");
    let body = body.into_inner();
    // A position can never be negative: clamp rather than storing junk.
    let position_seconds = body.position_seconds.max(0);
    match Episode::update_progress_by_yt_id(
        &data.pool,
        &yt_id.into_inner(),
        position_seconds,
        body.listened,
    ).await{
        // The request is fire-and-forget: the 204 status alone confirms the
        // write; no response body is needed.
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            error!("Error updating progress: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}
