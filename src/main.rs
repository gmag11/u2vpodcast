mod handlers;
mod models;
mod utils;

use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use std::{env::var, path::PathBuf, str::FromStr};
use tokio::{
    spawn,
    time::{sleep, Duration},
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use actix_files as af;
use models::{audios_dir, images_dir, AppState, Channel, Config, Error, User, Ytdlp};
use utils::throttle::init_throttle;
use utils::worker::do_the_work;

// Explicit origin allowlist for the production CORS policy. The configured
// `config.url` is validated at startup and appended by the CORS builder;
// this host is allowed so the SPA can fetch cover images. No trailing slash:
// origins are scheme://host[:port] only.
const YT_IMAGE_ORIGIN: &str = "https://yt3.googleusercontent.com";

// Validates that a CORS origin is a full `scheme://host[:port]` value with no
// path and no trailing slash, so the allowlist can never silently match
// everything or misparse.
fn validate_origin(origin: &str) -> Result<(), String> {
    let trimmed = origin.trim();
    let (scheme, rest) = trimmed.split_once("://").ok_or_else(|| {
        format!(
            "invalid CORS origin `{origin}`: missing `scheme://` (use e.g. https://podcasts.example.com)"
        )
    })?;
    if !matches!(scheme, "http" | "https") {
        return Err(format!(
            "invalid CORS origin `{origin}`: scheme must be `http` or `https`"
        ));
    }
    if rest.is_empty() || rest.contains('/') {
        return Err(format!(
            "invalid CORS origin `{origin}`: must not contain a path or trailing slash"
        ));
    }
    if rest.split(':').next().unwrap_or("").is_empty() {
        return Err(format!("invalid CORS origin `{origin}`: missing host"));
    }
    Ok(())
}

// Session keys must be at least 64 bytes: actix-session's `Key::from` panics
// otherwise. Validate at startup (both modes use the key) so a short
// `config.yml` key fails with a clear message instead of a panic (crash-safety).
fn validate_secret_key(key: &str) -> Result<(), String> {
    let trimmed = key.trim();
    if trimmed.len() < 64 {
        return Err(format!(
            "invalid `secret_key`: must be at least 64 bytes (got {}). \
             Generate one with e.g. `openssl rand -base64 48`",
            trimmed.len()
        ));
    }
    Ok(())
}

// Shared CORS builder: an explicit origin allowlist plus credential support in
// every mode. A wildcard origin combined with credentials would let any site
// read the API using the user's session cookie, so no mode may use
// `allow_any_origin()` (fix-dev-cors-with-credentials / api-cors-policy).
fn build_cors(origins: &[String]) -> Cors {
    let mut cors = Cors::default();
    for origin in origins {
        cors = cors.allowed_origin(origin);
    }
    cors.allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
        .allowed_header(header::CONTENT_TYPE)
        .expose_headers(&[header::CONTENT_DISPOSITION])
        .supports_credentials()
        .max_age(3600)
}

// The CORS allowlist for the current mode. Development restricts credentialed
// cross-origin requests to the configured URL plus the local SPA dev origins,
// mirroring the production posture instead of reflecting any origin.
fn cors_origins_for(config: &Config) -> Vec<String> {
    if config.production {
        vec![config.url.clone(), YT_IMAGE_ORIGIN.to_string()]
    } else {
        vec![
            config.url.clone(),
            format!("http://localhost:{}", config.port),
            format!("http://127.0.0.1:{}", config.port),
            YT_IMAGE_ORIGIN.to_string(),
        ]
    }
}

use actix_cors::Cors;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite},
    http::header,
    middleware::Logger,
    web::Data,
    App, HttpServer,
};

// Development-mode root directory. `CARGO_MANIFEST_DIR` only exists when the
// process was started through Cargo; fall back to the current working
// directory so a directly executed binary resolves a sensible local path
// instead of panicking (runtime-path-resolution).
fn dev_root() -> PathBuf {
    match var("CARGO_MANIFEST_DIR") {
        Ok(dir) => {
            info!("Using CARGO_MANIFEST_DIR `{dir}` as development root");
            PathBuf::from(dir)
        }
        Err(_) => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            info!(
                "CARGO_MANIFEST_DIR is unset; using current directory `{}` as development root",
                cwd.display()
            );
            cwd
        }
    }
}

static DDBB: &str = "u2vpodcast.db";
static MIGRATIONS_DIR: &str = "migrations";

#[actix_web::main]
async fn main() -> Result<(), Error> {
    let format = time::format_description::parse_borrowed::<2>(
        "[year]-[month padding:zero]-[day padding:zero]T[hour]:[minute]:[second]",
    )
    .expect("Can't parse timer");
    let offset_in_sec = chrono::Local::now().offset().local_minus_utc();
    let time_offset = time::UtcOffset::from_whole_seconds(offset_in_sec).unwrap();

    let timer = tracing_subscriber::fmt::time::OffsetTime::new(time_offset, format);
    let config = Config::load().await;
    let log_level = var("RUST_LOG").unwrap_or(config.log_level.clone());
    let log_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_timer(timer)
        //.with_thread_names(true)
        .with_filter(EnvFilter::from_str(&log_level).unwrap());

    tracing_subscriber::registry().with(log_layer).init();

    info!("Log level: {log_level}");

    let db_url = if var("RUST_ENV") == Ok("production".to_string()) {
        std::env::current_exe()?
            .parent()
            .unwrap()
            .join("db")
            .join(DDBB)
            .to_str()
            .unwrap()
            .to_string()
    } else {
        dev_root().join(DDBB).to_string_lossy().into_owned()
    };
    info!("DB url: {db_url}");
    // WAL + busy timeout: concurrent readers are not blocked by the single
    // writer and transient write contention waits instead of failing with
    // SQLITE_BUSY (db-pool-sizing). `create_if_missing` handles the DB file
    // creation that the removed `database_exists`/`create_database` block did.
    let db_options = SqliteConnectOptions::from_str(&db_url)
        .expect("valid sqlite URL derived from config")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(config.db_pool_max_connections.max(1))
        .connect_with(db_options)
        .await
        .expect("Pool failed");

    let migrations = if var("RUST_ENV") == Ok("production".to_string()) {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(MIGRATIONS_DIR)
    } else {
        dev_root().join(MIGRATIONS_DIR)
    };
    info!("{}", &migrations.display());

    Migrator::new(migrations).await?.run(&pool).await?;

    let sleep_time = config.sleep_time;
    let url = config.url.clone();
    let port = config.port;

    // Fail fast on a misconfigured CORS origin: a bad allowlist silently
    // deployed is worse than a loud refusal at startup.
    if config.production {
        validate_origin(&url).map_err(|e| Error::default(&e))?;
    }

    // The session key is used in every mode; actix-session panics on keys
    // shorter than 64 bytes, so validate up front (crash-safety).
    validate_secret_key(&config.secret_key).map_err(|e| Error::default(&e))?;

    // Validate the whole CORS allowlist up front in every mode: a dev origin
    // with a typo must not silently deploy a broader or broken policy.
    let cors_origins = cors_origins_for(&config);
    for origin in &cors_origins {
        validate_origin(origin).map_err(|e| Error::default(&e))?;
    }

    // Configure the single YouTube-connection throttle with the (optional)
    // cooldown from config.yml; the absent-key default applies via serde
    // (limit-youtube-concurrency / youtube-throttling).
    init_throttle(std::time::Duration::from_secs(config.cooldown_seconds));

    // When both admin credentials are set in config.yml, reseed the only user
    // from the configuration on every startup. When either is missing or empty,
    // the config credentials are ignored and the existing users table is kept
    // untouched, so authentication resolves against the stored database user.
    if config.admin_credentials_present() {
        let admin_username = config.admin_username.clone().unwrap_or_default();
        let admin_password = config.admin_password.clone().unwrap_or_default();
        User::delete_all(&pool)
            .await
            .expect("Cant delete existing users");
        User::default(&pool, &admin_username, &admin_password)
            .await
            .expect("Cant create admin user");
        info!("Admin reseeded from config.yml (seeded mode)");
    } else {
        info!("admin_username/admin_password not both set; ignoring config credentials and keeping existing users table");
    }

    // Backfill slugs and rename audio directories before the worker starts.
    // Use the shared audio path so the rename works outside Docker too
    // (runtime-path-resolution).
    Channel::migrate_slugs(&pool, audios_dir())
        .await
        .expect("Cant migrate slugs");

    // The image cache lives inside the `db` volume in the container
    // (/app/db/images) or a local `images` directory in development; create it
    // up front so the first cache write never fails on a missing directory
    // (channel-image-cache).
    tokio::fs::create_dir_all(images_dir())
        .await
        .expect("Cant create images cache directory");

    let pool2 = pool.clone();
    let worker_config = config.clone();
    spawn(async move {
        //let auth = HttpAuthentication::bearer(validator);
        loop {
            info!("**** Start updating yt-dlp ****");
            match Ytdlp::auto_update().await {
                Ok(()) => {}
                Err(e) => error!("{}", e),
            }
            info!("**** Finish updating yt-dlp ****");
            match do_the_work(&pool2, &worker_config).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Error doing the work: {e}");
                }
            }
            info!("Sleep time: {}", &sleep_time);
            sleep(Duration::from_secs(sleep_time * 3600)).await;
        }
    });

    let config2 = config.clone();
    HttpServer::new(move || {
        let appstate = AppState {
            config: config2.clone(),
            pool: pool.clone(),
        };
        let data = Data::new(appstate);
        let static_files = config.html_path.trim_end_matches('/').to_string();
        App::new()
            .wrap(Logger::default())
            .wrap(if config.production {
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(config.secret_key.as_bytes()).clone(),
                )
                .cookie_http_only(true)
                .cookie_same_site(SameSite::None)
                .cookie_secure(true)
                .session_lifecycle(
                    PersistentSession::default().session_ttl(time::Duration::days(7)),
                )
                .build()
            } else {
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(config.secret_key.as_bytes()).clone(),
                )
                .cookie_secure(false)
                .session_lifecycle(
                    PersistentSession::default().session_ttl(time::Duration::days(7)),
                )
                .build()
            })
            .wrap(build_cors(&cors_origins))
            .app_data(Data::clone(&data))
            .service(
                af::Files::new("/app", static_files.clone())
                    .index_file("index.html")
                    .default_handler(
                        af::NamedFile::open(
                            [static_files.clone(), "index.html".to_string()].join("/"),
                        )
                        .expect("index file should exist"),
                    ),
            )
            .configure(handlers::config_services)
    })
    .workers(2)
    .bind(("0.0.0.0", port))?
    .run()
    .await
    .map_err(|e| e.into())
}

#[cfg(test)]
mod cors_tests {
    use super::{validate_origin, validate_secret_key};

    #[test]
    fn valid_origins_are_accepted() {
        assert!(validate_origin("https://podcasts.example.com").is_ok());
        assert!(validate_origin("https://podcasts.example.com:8443").is_ok());
        assert!(validate_origin("http://localhost:6996").is_ok());
        assert!(validate_origin("https://yt3.googleusercontent.com").is_ok());
    }

    #[test]
    fn invalid_origins_are_rejected() {
        assert!(validate_origin("localhost").is_err());
        assert!(validate_origin("http://").is_err());
        assert!(validate_origin("https://podcasts.example.com/").is_err());
        assert!(validate_origin("https://podcasts.example.com/path").is_err());
        assert!(validate_origin("ftp://example.com").is_err());
    }

    #[test]
    fn dev_local_origins_are_accepted() {
        assert!(validate_origin("http://localhost:5173").is_ok());
        assert!(validate_origin("http://localhost:6996").is_ok());
        assert!(validate_origin("http://127.0.0.1:6996").is_ok());
        assert!(validate_origin("http://[::1]:6996").is_ok());
    }

    #[test]
    fn short_secret_key_is_rejected() {
        assert!(validate_secret_key("").is_err());
        assert!(validate_secret_key("short").is_err());
        assert!(validate_secret_key(&"x".repeat(63)).is_err());
    }

    #[test]
    fn valid_secret_key_is_accepted() {
        let key = "x".repeat(64);
        assert!(validate_secret_key(&key).is_ok());
        assert!(validate_secret_key(&format!("  {}  ", key)).is_ok());
    }
}

#[cfg(test)]
mod error_serialization_tests {
    use crate::models::Error;
    use actix_web::http::StatusCode;

    #[test]
    fn error_without_status_serializes_as_500() {
        let error = Error::default("boom");
        let value = serde_json::to_value(&error).expect("serializing Error must not panic");
        assert_eq!(value["status_code"], 500);
        assert_eq!(value["details"], "boom");
    }

    #[test]
    fn error_with_status_serializes_it() {
        let error = Error::new_with_status_code("nope", StatusCode::BAD_REQUEST);
        let value = serde_json::to_value(&error).expect("serializing Error must not panic");
        assert_eq!(value["status_code"], 400);
    }
}
