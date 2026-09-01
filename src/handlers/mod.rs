mod channels;
mod config;
mod episodes;
mod feed;
mod login;
mod logout;
mod media;
mod options;
mod playlists;
mod status;
mod users;

use actix_files as af;
use actix_web::web;
use tracing::info;

use super::models::{images_dir, AppState, Credentials};
use super::utils::middleware::{RequireSession, SessionOrBasicAuth};
use feed::web_feed;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    info!("Configuring routes...");
    cfg.service(
        web::scope("")
            .service(web::redirect("/", "/app/"))
            .configure(web_feed)
            .service(
                web::scope("/api").service(
                    web::scope("/1.0")
                        .service(web::resource("/logout/").route(web::get().to(logout::get_logout)))
                        .service(web::resource("/status/").route(web::get().to(status::get_status)))
                        .service(web::resource("/login/").route(web::post().to(login::post_login)))
                        .service(
                            web::resource("/session/").route(web::get().to(login::get_session)),
                        )
                        .service(
                            web::scope("")
                                .wrap(RequireSession)
                                .configure(users::api_users)
                                .configure(options::api_options)
                                .configure(playlists::api_playlists)
                                .service(channels::read)
                                .service(channels::read_all)
                                .service(episodes::read_with_pagination)
                                .service(episodes::read_all)
                                .service(episodes::read_progress)
                                .service(episodes::update_progress)
                                .service(episodes::update_favorite)
                                .service(episodes::refresh_sponsorblock)
                                .service(channels::create)
                                .service(channels::update_episodes)
                                .service(channels::refresh_image)
                                .service(channels::update)
                                .service(channels::update_playback_speed)
                                .service(channels::delete)
                                .service(config::get_config),
                        ),
                ),
            )
            .service(
                web::scope("/media")
                    .wrap(SessionOrBasicAuth)
                    .route("/{path:.*}", web::get().to(media::serve_media))
                    .route("/{path:.*}", web::head().to(media::serve_media)),
            )
            .service(
                web::scope("/images")
                    .wrap(SessionOrBasicAuth)
                    .service(af::Files::new("", images_dir())),
            ),
    );
}
