-- SQLite cannot drop arbitrary columns from an existing table, so the down
-- migration recreates `episodes` without the playback-progress columns and
-- copies every existing row across.
PRAGMA foreign_keys=OFF;

CREATE TABLE episodes_new(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    channel_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    yt_id TEXT NOT NULL,
    webpage_url TEXT NOT NULL,
    published_at DATETIME NOT NULL,
    duration TEXT NOT NULL,
    image TEXT NOT NULL DEFAULT '',
    listen BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    UNIQUE(channel_id, yt_id)
);

INSERT INTO episodes_new (id, channel_id, title, description, yt_id, webpage_url, published_at, duration, image, listen, created_at, updated_at)
    SELECT id, channel_id, title, description, yt_id, webpage_url, published_at, duration, image, listen, created_at, updated_at FROM episodes;

DROP TABLE episodes;

ALTER TABLE episodes_new RENAME TO episodes;

PRAGMA foreign_keys=ON;