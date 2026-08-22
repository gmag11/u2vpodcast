mod login;
mod logout;
mod status;
mod channels;
mod episodes;
mod users;
mod options;
mod feed;
mod config;


use actix_web::web;
use tracing::info;
use actix_files as af;

use super::models::{
    Credentials,
    AppState,
    audios_dir,
    images_dir,
};
use super::utils::middleware::{
    RequireSession,
    SessionOrBasicAuth,
};
use feed::web_feed;


pub fn config_services(cfg: &mut web::ServiceConfig) {
    info!("Configuring routes...");
    cfg.service(
        web::scope("")
            .service(
                web::redirect("/", "/app/")
            )
            .configure(web_feed)
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/1.0")
                            .service(
                                web::resource("/logout/")
                                    .route(web::get().to(logout::get_logout)))
                            .service(
                                web::resource("/status/")
                                    .route(web::get().to(status::get_status)))
                            .service(
                                web::resource("/login/")
                                    .route(web::post().to(login::post_login)))
                            .service(
                                web::resource("/session/")
                                    .route(web::get().to(login::get_session)))
                            .service(
                                web::scope("")
                                    .wrap(RequireSession)
                                    .configure(users::api_users)
                                    .configure(options::api_options)
                                    .service(channels::read)
                                    .service(channels::read_with_pagination)
                                    .service(episodes::read_with_pagination)
                                    .service(episodes::read_all)
                                    .service(channels::create)
                                    .service(channels::update_episodes)
                                    .service(channels::refresh_image)
                                    .service(channels::update)
                                    .service(channels::delete)
                                    .service(config::get_config)
                            )
                    )
            )
            .service(
                web::scope("/media")
                    .wrap(SessionOrBasicAuth)
                    .service(af::Files::new("", audios_dir()))
            )
            .service(
                web::scope("/images")
                    .wrap(SessionOrBasicAuth)
                    .service(af::Files::new("", images_dir()))
            )
    );
}
