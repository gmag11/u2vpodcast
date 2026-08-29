mod appstate;
pub mod channel;
pub mod config;
mod episode;
mod error;
mod param;
mod playlist;
pub mod response;
mod role;
mod sponsorblock;
mod user;
mod ytdlp;
pub mod ytinfo;

use chrono::{DateTime, Utc};

pub use appstate::AppState;
pub use channel::{Channel, NewChannel, UpdateChannel};
pub use config::{
    audios_dir, cookies_file, images_dir, ytdlp_path, Config, SUPPORTED_SPONSORBLOCK_CATEGORIES,
};
pub use episode::{Episode, EpisodeProgress};
pub use error::Error;
pub use param::Param;
pub use playlist::PlaylistItem;
pub use response::{CResponse, CustomResponse};
pub use sponsorblock::{EpisodeSponsorBlockSegment, SponsorBlockCache, SponsorBlockSegment};
pub use user::{from_session, Credentials, NewUser, User};
pub use ytdlp::{YtVideo, Ytdlp};
use ytinfo::{cache_image, YTInfo};

pub fn default_datetime() -> DateTime<Utc> {
    Utc::now()
}
