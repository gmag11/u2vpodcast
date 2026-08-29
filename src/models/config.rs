use serde::Deserialize;
use std::{collections::HashSet, path::Path};
use tokio::fs::read_to_string;
use tracing::{debug, info};

pub const SUPPORTED_SPONSORBLOCK_CATEGORIES: [&str; 8] = [
    "sponsor",
    "selfpromo",
    "interaction",
    "intro",
    "outro",
    "preview",
    "music_offtopic",
    "filler",
];

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub production: bool,
    pub url: String,
    pub port: u16,
    pub sleep_time: u64,
    pub per_page: i64,
    pub secret_key: String,
    pub admin_username: Option<String>,
    pub admin_password: Option<String>,
    #[serde(default = "default_with_authentication")]
    pub with_authentication: bool,
    #[serde(default = "default_html_path")]
    pub html_path: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_db_pool_max_connections")]
    pub db_pool_max_connections: u32,
    #[serde(default = "default_cooldown_seconds")]
    pub cooldown_seconds: u64,
    #[serde(default)]
    pub sponsorblock_enabled: bool,
    #[serde(default = "default_sponsorblock_rejected_categories")]
    pub sponsorblock_rejected_categories: Vec<String>,
}

fn default_sponsorblock_rejected_categories() -> Vec<String> {
    vec!["sponsor".to_string()]
}

fn default_cooldown_seconds() -> u64 {
    3
}

fn default_db_pool_max_connections() -> u32 {
    5
}

fn default_with_authentication() -> bool {
    true
}

fn default_log_level() -> String {
    "INFO".to_string()
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

// Cache directory for channel cover images. The `db` volume is already mounted
// at `/app/db` in the container (see docker-compose.yml), so `/app/db/images`
// persists across container recreation without any new volume. We only probe
// the parent mount (`/app/db`) rather than the images subdirectory itself: the
// `db` volume mount point always exists, while `images/` may legitimately not
// have been created yet (channel-image-cache). Outside Docker, fall back to a
// plain local `images` directory for development.
pub fn images_dir() -> &'static str {
    if Path::new("/app/db").is_dir() {
        "/app/db/images"
    } else {
        "images"
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
    if Path::new("/app/cookies.txt").exists() {
        "/app/cookies.txt"
    } else if Path::new("cookies.txt").exists() {
        "cookies.txt"
    } else {
        ""
    }
}

impl Config {
    fn from_yaml(content: &str) -> Result<Self, String> {
        let config: Self = serde_yaml::from_str(content)
            .map_err(|error| format!("Can't process config file `config.yml`: {error}"))?;
        config.validate()
    }

    fn validate(mut self) -> Result<Self, String> {
        let configured = self
            .sponsorblock_rejected_categories
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        if let Some(invalid) = configured
            .iter()
            .find(|category| !SUPPORTED_SPONSORBLOCK_CATEGORIES.contains(category))
        {
            return Err(format!(
                "invalid SponsorBlock category `{invalid}` in `sponsorblock_rejected_categories`"
            ));
        }
        self.sponsorblock_rejected_categories = SUPPORTED_SPONSORBLOCK_CATEGORIES
            .iter()
            .filter(|category| configured.contains(**category))
            .map(|category| (*category).to_string())
            .collect();
        Ok(self)
    }

    pub fn admin_credentials_present(&self) -> bool {
        let username = self
            .admin_username
            .as_deref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        let password = self
            .admin_password
            .as_deref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        username.is_some() && password.is_some()
    }

    pub async fn load() -> Self {
        info!("load");
        let content = read_to_string("config.yml")
            .await
            .expect("Can't read config file `config.yml`");
        debug!("Content: {content}");
        Self::from_yaml(&content).unwrap_or_else(|error| panic!("{error}"))
    }
}

// Minimal valid config for tests that need an `AppState` (handlers require
// one even when they only touch the pool). Test-only, so it lives here next
// to the same YAML helper the config tests use.
#[cfg(test)]
pub(crate) fn test_config() -> Config {
    let yaml = format!(
        "production: false\nurl: http://localhost:6996\nport: 6996\nsleep_time: 1\nper_page: 3\nsecret_key: {}\n",
        "x".repeat(64)
    );
    Config::from_yaml(&yaml).expect("test config must parse")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaml(extra: &str) -> String {
        format!(
            "production: false\nurl: http://localhost:6996\nport: 6996\nsleep_time: 1\nper_page: 3\nsecret_key: {}\n{extra}",
            "x".repeat(64)
        )
    }

    #[test]
    fn sponsorblock_defaults_to_disabled_and_sponsor_only() {
        let config = Config::from_yaml(&yaml("")).unwrap();
        assert!(!config.sponsorblock_enabled);
        assert_eq!(config.sponsorblock_rejected_categories, ["sponsor"]);
    }

    #[test]
    fn sponsorblock_accepts_enabled_empty_and_normalized_categories() {
        let empty = Config::from_yaml(&yaml(
            "sponsorblock_enabled: true\nsponsorblock_rejected_categories: []\n",
        ))
        .unwrap();
        assert!(empty.sponsorblock_enabled);
        assert!(empty.sponsorblock_rejected_categories.is_empty());

        let normalized = Config::from_yaml(&yaml(
            "sponsorblock_enabled: true\nsponsorblock_rejected_categories: [intro, sponsor, intro]\n",
        ))
        .unwrap();
        assert_eq!(
            normalized.sponsorblock_rejected_categories,
            ["sponsor", "intro"]
        );
    }

    #[test]
    fn sponsorblock_disabled_selection_is_valid_but_does_not_enable_it() {
        let config = Config::from_yaml(&yaml(
            "sponsorblock_enabled: false\nsponsorblock_rejected_categories: [outro]\n",
        ))
        .unwrap();
        assert!(!config.sponsorblock_enabled);
        assert_eq!(config.sponsorblock_rejected_categories, ["outro"]);
    }

    #[test]
    fn checked_in_sample_config_parses() {
        Config::from_yaml(include_str!("../../config.yml")).expect("sample config must parse");
    }

    #[test]
    fn sponsorblock_rejects_unknown_categories() {
        let error =
            Config::from_yaml(&yaml("sponsorblock_rejected_categories: [sponser]\n")).unwrap_err();
        assert!(error.contains("sponser"));
        assert!(error.contains("invalid SponsorBlock category"));
    }
}
