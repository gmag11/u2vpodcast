CREATE TABLE sponsorblock_cache (
    episode_id INTEGER PRIMARY KEY NOT NULL,
    segments_json TEXT NOT NULL,
    snapshot_hash TEXT NOT NULL,
    checked_at DATETIME NOT NULL,
    processed_filename TEXT,
    processed_duration REAL,
    last_error TEXT,
    last_error_at DATETIME,
    FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE
);