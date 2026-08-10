use actix_web::{
    Responder,
    get,
    web::{
        Path,
        Data,
    },
};
use actix_session::Session;
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
