use actix_web::{
    Responder,
    web::{
        Path,
        Data,
        Json,
        Query,
    },
    get,
    put,
    post,
    delete,
};
use actix_session::Session;
use serde::Deserialize;
use tracing::{
    info,
    debug,
    error,
};

use super::{
    AppState,
    super::{
        models::{
            audios_dir,
            images_dir,
            CResponse,
            Channel,
            NewChannel,
            UpdateChannel,
        },
        utils::worker::update_channel as refresh_channel,
    },
};

#[derive(Deserialize)]
struct Page{
    page: Option<i64>,
}


#[get("/channels/")]
async fn read_with_pagination(
    data: Data<AppState>,
    session: Session,
    page: Query<Page>,
) -> impl Responder{
    info!("read_all");
    let page = page.page.unwrap_or(1);
    let per_page = data.config.per_page;
    match Channel::read_with_pagination(&data.pool, page, per_page).await{
        Ok(channels) => Ok(CResponse::ok(session, channels)),
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        },
    }
}

#[post("/channels/")]
async fn create(
    data: Data<AppState>,
    session: Session,
    channel: Json<NewChannel>,
) -> impl Responder {
    info!("create");
    match Channel::new(&data.pool, channel.into_inner()).await{
            Ok(channel) => {
                let pool = data.pool.clone();
                let config = data.config.clone();
                let id = channel.id;
                actix_web::rt::spawn(async move{
                    if let Err(e) = refresh_channel(&pool, id, &config).await{
                        error!("Cant refresh new channel {}: {}", id, e);
                    }
                });
                Ok(CResponse::ok(session, channel))
            },
            Err(mut e) => {
                error!("Error: {e}");
                e.set_session(session);
                Err(e)
            },
        }
}

#[post("/channels/{channel}/update/")]
async fn update_episodes(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
) -> impl Responder {
    info!("update_episodes");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await{
        Ok(channel) => {
            let pool = data.pool.clone();
            let config = data.config.clone();
            let id = channel.id;
            actix_web::rt::spawn(async move{
                if let Err(e) = refresh_channel(&pool, id, &config).await{
                    error!("Cant refresh channel {}: {}", id, e);
                }
            });
            Ok(CResponse::ok(session, channel))
        },
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        },
    }
}

#[post("/channels/{channel}/image/")]
async fn refresh_image(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
) -> impl Responder {
    info!("refresh_image");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await{
        Ok(channel) => {
            let url = channel.url.clone();
            match Channel::update_image(&data.pool, channel.id, &url).await{
                Ok(channel) => Ok(CResponse::ok(session, channel)),
                Err(mut e) => {
                    error!("Error: {e}");
                    e.set_session(session);
                    Err(e)
                },
            }
        },
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        },
    }
}

#[put("/channels/{channel}/")]
async fn update(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
    channel: Json<UpdateChannel>,
) -> impl Responder {
    info!("update");
    let key = path.into_inner();
    let mut channel = channel.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await{
        Ok(existing) => {
            channel.id = existing.id;
            match Channel::update(&data.pool, &channel).await{
                Ok(channel) => Ok(CResponse::ok(session, channel)),
                Err(mut e) => {
                    error!("Error: {e}");
                    e.set_session(session);
                    Err(e)
                },
            }
        },
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        },
    }
}


#[get("/channels/{channel}/")]
async fn read(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
) -> impl Responder{
    info!("read");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await{
            Ok(channel) => Ok(CResponse::ok(session, channel)),
            Err(mut e) => {
                error!("Error: {e}");
                e.set_session(session);
                Err(e)
            },
        }
}
#[delete("/channels/{channel}/")]
async fn delete(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
) -> impl Responder{
    info!("delete");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await{
            Ok(channel) => {
                let folder = audios_dir();
                // Ownership guard: only remove the audio directory when no other
                // channel row references the same slug path. Post-migration the
                // slug is unique, but the guard covers stale/edge states so a
                // shared directory can never be wiped while another channel's
                // rows survive.
                match Channel::count_by_slug(&data.pool, &channel.slug).await {
                    Ok(count) if count > 1 => {
                        error!(
                            "Channel slug `{}` still referenced by {} channels; skipping directory removal to avoid wiping another channel's audio",
                            &channel.slug, count
                        );
                    }
                    _ => {
                        info!("Remove directory {}/{}", folder, &channel.slug);
                        match tokio::fs::remove_dir_all(format!("{}/{}", folder, channel.slug))
                            .await {
                            Ok(_) => debug!("Removed directorio {}/{}", folder, &channel.slug),
                            Err(e) => error!("Can't remove directory {}/{}: {}", folder, &channel.slug, e),
                        };
                        // Remove the cached cover image alongside the audio
                        // directory (channel-image-cache). Missing file is not
                        // an error: the channel may never have had a cached
                        // image.
                        match tokio::fs::remove_file(
                            format!("{}/{}.jpg", images_dir(), channel.slug)
                        ).await {
                            Ok(_) => debug!("Removed cached image for {}", &channel.slug),
                            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
                            Err(e) => error!("Can't remove cached image for {}: {}", &channel.slug, e),
                        };
                    }
                }
                match Channel::delete(&data.pool, channel.id).await{
                    Ok(channel) => Ok(CResponse::ok(session, channel)),
                    Err(mut e) => {
                        error!("Error: {e}");
                        e.set_session(session);
                        Err(e)
                    },
                }
        },
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
            },
    }
}
