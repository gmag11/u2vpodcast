use std::path::Path;
use serde::Deserialize;
use tokio::fs::read_to_string;
use tracing::{info, debug};

#[derive(Debug, Clone, Deserialize)]
pub struct Config{
    pub production: bool,
    pub title: String,
    pub url: String,
    pub port: u16,
    pub sleep_time: u64,
    pub per_page: i64,
    pub secret_key: String,
    pub admin_username: String,
    pub admin_password: String,
    #[serde(default = "default_with_authentication")]
    pub with_authentication: bool,
    #[serde(default = "default_html_path")]
    pub html_path: String,
}

fn default_with_authentication() -> bool {
    true
}

fn default_html_path() -> String {
    if Path::new("/app/html/index.html").exists() {
        "/app/html".to_string()
    } else {
        "frontend/dist".to_string()
    }
}

pub fn audios_dir() -> &'static str {
    if Path::new("/app/audios").is_dir() {
        "/app/audios"
    } else {
        "audios"
    }
}

pub fn ytdlp_path() -> &'static str {
    if Path::new("/app/.local/bin/yt-dlp").exists() {
        "/app/.local/bin/yt-dlp"
    } else {
        "yt-dlp"
    }
}

pub fn cookies_file() -> &'static str {
    if Path::new("/app/cookies-cp.txt").exists() {
        "/app/cookies-cp.txt"
    } else if Path::new("cookies-cp.txt").exists() {
        "cookies-cp.txt"
    } else {
        ""
    }
}

impl Config {
    pub async fn load() -> Self {
        info!("load");
        let content = read_to_string("config.yml")
            .await
            .expect("Can't read config file `config.yml`");
        debug!("Content: {content}");
        serde_yaml::from_str(&content)
            .expect("Can't process config file `config.yml`")
    }
}
