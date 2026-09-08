use super::config::Config;
use sqlx::{Pool, Sqlite};

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Pool<Sqlite>,
}
