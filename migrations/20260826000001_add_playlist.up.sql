CREATE TABLE IF NOT EXISTS playlist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    episode_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    added_at DATETIME NOT NULL,
    UNIQUE(episode_id)
);