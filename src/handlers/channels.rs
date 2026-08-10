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
    super::models::{
        CResponse,
        Channel,
        NewChannel,
        UpdateChannel,
    },
};

static FOLDER: &str = "/app/audios";


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
            Ok(channel) => Ok(CResponse::ok(session, channel)),
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
                info!("Remove directory {}/{}", FOLDER, &channel.slug);
                match tokio::fs::remove_dir_all(format!("{}/{}", FOLDER, &channel.slug))
                    .await {
                    Ok(_) => debug!("Removed directorio {}/{}", FOLDER, &channel.slug),
                    Err(e) => error!("Can't remove directory {}/{}: {}", FOLDER, &channel.slug, e),
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
