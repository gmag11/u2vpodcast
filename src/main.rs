mod handlers;
mod models;
mod utils;

use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use std::{env::var, path::PathBuf, str::FromStr, sync::atomic::{AtomicU64, Ordering}};
use tokio::{
    spawn,
    time::{sleep, Duration},
};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use actix_files as af;
use models::{audios_dir, images_dir, AppState, Channel, Config, Error, User, Ytdlp};
use utils::throttle::init_throttle;
use utils::worker::do_the_work;

// Explicit origin allowlist for the production CORS policy. The configured
// `config.url` is validated at startup and appended by the CORS builder;
// this host is allowed so the SPA can fetch cover images. No trailing slash:
// origins are scheme://host[:port] only.
// (The allowlist lives in `Config::cors_origins`; shared with the CSRF check.)

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

// True when the session key is still the placeholder shipped in the sample
// config.yml. The placeholder is a known value: reusing it in production lets
// anyone with the repository forge session cookies.
fn is_placeholder_secret(key: &str) -> bool {
    key.contains("REPLACE_THIS")
}

// Production requires the canonical origin (and therefore the Secure session
// cookie) to be https; plain http breaks login silently behind the proxy.
fn requires_https(production: bool, url: &str) -> bool {
    production && url.starts_with("http://")
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

// Content-Security-Policy for the SPA and its API responses (defense in depth;
// the frontend has no `v-html` on network data). `script-src` allows only
// same-origin scripts plus the sha256 of the inline theme bootstrapper in
// frontend/index.html — if that inline script changes, recompute its hash.
// `style-src 'unsafe-inline'` is required by Vue inline style bindings; Google
// Fonts is loaded from the stylesheet link in index.html. `frame-ancestors`
// blocks clickjacking; `media-src`/`connect-src` keep playback and API on the
// same origin.
const CSP: &str = "default-src 'self'; \
     script-src 'self' 'sha256-z8kQzMAgtRsSW0cXM+XkZvyyilr5edUUaKINvHCx0ss='; \
     style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
     font-src 'self' https://fonts.gstatic.com; \
     img-src 'self' data: https:; \
     media-src 'self' blob:; \
     connect-src 'self'; \
     object-src 'none'; \
     frame-ancestors 'none'; \
     base-uri 'self'; \
     form-action 'self'";

// Hardening response headers applied to every response (static files, media
// and API alike). CSP only applies to HTML/document responses; the headers are
// harmless on JSON/media payloads.
fn security_headers() -> DefaultHeaders {
    DefaultHeaders::new()
        .add((header::CONTENT_SECURITY_POLICY, CSP))
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("X-Frame-Options", "DENY"))
        .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
        .add(("Permissions-Policy", "geolocation=(), microphone=(), camera=()"))
        .add(("X-XSS-Protection", "0"))
}

// The CORS allowlist for the current mode. Development restricts credentialed
// cross-origin requests to the configured URL plus the local SPA dev origins,
// mirroring the production posture instead of reflecting any origin.
// Shared with the CSRF origin check via `Config::cors_origins`.

use actix_cors::Cors;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite},
    http::header,
    middleware::{DefaultHeaders, Logger},
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

fn unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// True when the process runs as a real deployment (RUST_ENV=production, set by
// the docker-compose files). Local `cargo run` development leaves it unset, and
// the deployment-only safety guards below relax to warnings in that case so the
// sample config (placeholder key, http url) keeps working locally.
fn deployment_mode(rust_env: Option<&str>) -> bool {
    rust_env == Some("production")
}

fn is_deployment_runtime() -> bool {
    deployment_mode(var("RUST_ENV").ok().as_deref())
}

// True when the periodic auto-update should run: at least `interval_secs` have
// elapsed since the last run (`last`), or it never ran.
fn update_due(now: u64, last: u64, interval_secs: u64) -> bool {
    now.saturating_sub(last) >= interval_secs
}

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
        .busy_timeout(std::time::Duration::from_secs(5))
        // Enforce foreign keys explicitly. SQLite leaves them off by default
        // per connection and sqlx turns them on by default — this makes the
        // requirement explicit so the `ON DELETE CASCADE` on
        // `sponsorblock_cache.episode_id` cannot silently regress.
        .foreign_keys(true);

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

    // The sample config ships a placeholder session key; a deployment that
    // reuses it lets anyone with the repository forge session cookies. Fail
    // fast only in a real deployment (RUST_ENV=production); in local
    // development the sample config keeps working with a warning.
    if is_placeholder_secret(&config.secret_key) {
        if is_deployment_runtime() {
            return Err(Error::default(
                "config.yml `secret_key` is still the sample placeholder — generate a random \
                 64-byte key for this deployment before going live \
                 (openssl rand -base64 48 | tr -d '\\n')",
            ));
        }
        warn!(
            "config.yml `secret_key` is the sample placeholder — generate a random 64-byte key \
             for this deployment before going live (openssl rand -base64 48 | tr -d '\\n')"
        );
    }

    // The production session cookie is `Secure`; without TLS in front of the
    // app the browser never sends it and login silently breaks. Fail fast in a
    // real deployment; local development may run over plain http.
    if requires_https(is_deployment_runtime(), &config.url) {
        return Err(Error::default(
            "config.yml `url` must use https:// in production mode: the Secure \
             session cookie requires HTTPS (and it is the CORS/feed origin). \
             Terminate TLS at the reverse proxy and set `url` to the public \
             https origin.",
        ));
    }

    // Validate the whole CORS allowlist up front in every mode: a dev origin
    // with a typo must not silently deploy a broader or broken policy.
    let cors_origins = config.cors_origins();
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
        // yt-dlp auto-update cadence: at most once a day per process. The
        // worker loop runs every `sleep_time` hours; without a gate it would
        // hit the update channel on every cycle. `LAST_AUTO_UPDATE` is
        // per-process state (survives only until restart), which is enough to
        // stop the loop hammering the update source.
        static LAST_AUTO_UPDATE: AtomicU64 = AtomicU64::new(0);
        const AUTO_UPDATE_INTERVAL_SECS: u64 = 24 * 3600;
        loop {
            let now = unix_secs();
            let last = LAST_AUTO_UPDATE.load(Ordering::Relaxed);
            if update_due(now, last, AUTO_UPDATE_INTERVAL_SECS) {
                info!("**** Start updating yt-dlp ****");
                match Ytdlp::auto_update().await {
                    Ok(()) => {}
                    Err(e) => error!("{}", e),
                }
                LAST_AUTO_UPDATE.store(now, Ordering::Relaxed);
            } else {
                info!("Skipping yt-dlp auto-update (last run < 24h ago)");
            }
            info!("**** Start updating channels ****");
            match do_the_work(&pool2, &worker_config).await {
                Ok(()) => {}
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
            .wrap(security_headers())
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
mod deployment_guard_tests {
    use super::{deployment_mode, is_placeholder_secret, requires_https, update_due};

    #[test]
    fn placeholder_secret_is_detected() {
        assert!(is_placeholder_secret("REPLACE_THIS_WITH_YOUR_OWN_RANDOM_64_BYTE_SECRET_KEY_1234567890_"));
        assert!(is_placeholder_secret("prefix REPLACE_THIS suffix"));
        assert!(!is_placeholder_secret(&"x".repeat(64)));
        assert!(!is_placeholder_secret(""));
    }

    #[test]
    fn deployment_mode_detects_production_env_only() {
        assert!(deployment_mode(Some("production")));
        assert!(!deployment_mode(Some("development")));
        assert!(!deployment_mode(Some("")));
        assert!(!deployment_mode(None));
    }

    #[test]
    fn production_requires_https_url() {
        assert!(requires_https(true, "http://localhost:6996"));
        assert!(requires_https(true, "http://podcasts.example.com"));
        assert!(!requires_https(true, "https://podcasts.example.com"));
        assert!(!requires_https(false, "http://localhost:6996"));
    }

    #[test]
    fn auto_update_runs_at_most_once_per_interval() {
        let interval = 24 * 3600;
        // Never ran (last = 0, far in the past) -> due.
        assert!(update_due(1_700_000_000, 0, interval));
        // Ran recently -> not due.
        assert!(!update_due(100, 50, interval));
        assert!(!update_due(interval - 1, 0, interval));
        // Interval elapsed -> due.
        assert!(update_due(interval, 0, interval));
        assert!(update_due(interval + 10, 0, interval));
    }

    #[test]
    fn shipped_sample_config_still_uses_placeholder_key() {
        // The checked-in sample config.yml must keep a 64-byte placeholder
        // secret_key so copy-paste deployments cannot silently run with a
        // known signing key (main() fails fast in production on it).
        let sample = include_str!("../config.yml");
        assert!(sample.contains("REPLACE_THIS"));
        assert!(sample.contains("secret_key:"));
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
