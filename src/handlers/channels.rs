use actix_web::{
    Responder,
    web::{
        Path,
        Data,
        Json,
    },
    get,
    put,
    post,
    delete,
};
use actix_session::Session;
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
            CResponse,
            Channel,
            NewChannel,
            UpdateChannel,
        },
        utils::worker::update_channel as refresh_channel,
    },
};


#[get("/channels/")]
async fn read_with_pagination(
    data: Data<AppState>,
    session: Session,
) -> impl Responder{
    info!("read_all");
    match Channel::read_all(&data.pool).await{
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
                let id = channel.id;
                actix_web::rt::spawn(async move{
                    if let Err(e) = refresh_channel(&pool, id).await{
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
            let id = channel.id;
            actix_web::rt::spawn(async move{
                if let Err(e) = refresh_channel(&pool, id).await{
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
                info!("Remove directory {}/{}", folder, &channel.slug);
                match tokio::fs::remove_dir_all(format!("{}/{}", folder, &channel.slug))
                    .await {
                    Ok(_) => debug!("Removed directorio {}/{}", folder, &channel.slug),
                    Err(e) => error!("Can't remove directory {}/{}: {}", folder, &channel.slug, e),
                };
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
