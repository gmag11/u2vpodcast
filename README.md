<div align="center">
<h1 align="center">u2vpodcast</h1>
<br />
<img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg" /><br>
<br>
A service to create your podcasts from your favourites YouTube Channels
</div>

---

### Installation

With docker-compose

* Change `docker-compose` and copy `sample.env` to `.env`. Change `.env` as you need.
* Create `cookies.txt` from your YouTube cookies.

### Create cookies.txt

#### How do you cookies to work

In order to extract cookies from browser use any conforming browser extension for exporting cookies.

For example,

* Chrome => https://chrome.google.com/webstore/detail/get-cookiestxt/bgaddhkoddajcdgocldbbfleckgcbcid/
* Firefox => https://addons.mozilla.org/en-US/firefox/addon/cookies-txt/

#### Correct file ownership

The container runs as the `app` user (UID `10001`). `yt-dlp` reads the cookies from `cookies.txt` and also writes back to it on every download, so the file must be writable by the container user or downloads fail with a `PermissionError`.

Since `cookies.txt` is bind-mounted from the host (`./cookies.txt:/app/cookies.txt`), make it writable on the host:

```bash
chmod 666 cookies.txt
```

Alternatively, make the container user the owner:

```bash
sudo chown 10001:10001 cookies.txt   # Docker (rootful Podman)
sudo chown <your-subuid>:<your-subuid> cookies.txt  # rootless Podman
```

When running rootless Podman, the container UID is mapped to a subordinate UID of your user (check `/etc/subuid`). After a rootless container changes the ownership of `cookies.txt`, edit it from the host with `podman unshare chown <your-uid>:<your-gid> cookies.txt`.

### Configuration

You need to modify `config.yml`. Change the params as you need, and add all the channels and YouTube list that you want

#### `config.yml` options

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `production` | boolean | — | When `true`, runs in production mode: session cookies are marked secure (`SameSite=None`, `Secure`) and CORS is restricted to the configured `url`. In development (`false`), cookies and CORS are relaxed. |
| `title` | string | — | Name of the service (informational). |
| `url` | string | — | Public URL/host of the service. Used as allowed CORS origin in production and as base URL for the RSS feeds. |
| `port` | integer | — | TCP port the HTTP server listens on. |
| `sleep_time` | number | — | Hours between each background update cycle (checks for new videos and runs `yt-dlp`). |
| `per_page` | integer | — | Number of items shown per page in the channel and episode listings. |
| `secret_key` | string | — | Secret used to sign session cookies. Generate a strong random value and keep it secret. |
| `admin_username` | string | — | Username of the admin user. Only used when both `admin_username` and `admin_password` are set: in that case the admin is recreated from these values on every startup. |
| `admin_password` | string | — | Password of the admin user. Only used when both `admin_username` and `admin_password` are set. The database never stores it in plaintext, only its hash. |
| `with_authentication` | boolean | `true` | When `true`, the RSS feed (`/channels/{channel_id}/feed.xml`) and the media files (`/media/**`) require HTTP Basic Auth using the admin credentials. When `false`, they are served without authentication. |
| `cooldown_seconds` | integer | `3` | Pause (in seconds) between consecutive YouTube connections imposed by the single-connection throttle: metadata fetches, cover image fetches, and every `yt-dlp` run are serialized and separated by this gap. |
| `sponsorblock_enabled` | boolean | `false` | Master switch for SponsorBlock. When false, retrieval, reconciliation, processing, API data, refresh controls, playback skipping, timeline markers, and processed feed media are all bypassed. Existing installations must set this to `true` to retain SponsorBlock behavior after upgrading. |
| `sponsorblock_rejected_categories` | string list | `[sponsor]` | Categories cut from derived feed audio and skipped by the web player when enabled. Supported values are `sponsor`, `selfpromo`, `interaction`, `intro`, `outro`, `preview`, `music_offtopic`, and `filler`. Duplicates have no additional effect; unsupported values prevent startup. An explicit `[]` rejects nothing while still showing all available categories. |

If only one of `admin_username` and `admin_password` is set (or both are missing), both are **ignored**: the `users` table is left untouched on startup and the service authenticates against the admin account already stored in the database. To adopt this mode, log in once with the seeded credentials so the user row exists, then remove both keys from `config.yml` and restart.

**Sessions and restarts:** in seeded mode (both keys present) the admin user is recreated on every startup with a **new database row id**, so existing session cookies stop validating (the session middleware rechecks the user on each request) and you must log in again after every container restart. In the persistent mode above, the `users` table is never rewritten and login sessions survive restarts. Keep `secret_key` unchanged too — changing it invalidates every signed session cookie regardless of mode.

### Usage

```
docker-compose up -d
```
If you need to run [u2vpodcast](https://github.com/atareao/u2vpodcast) behind reverse proxy, like [caddy](https://github.com/caddyserver/caddy), run:

```
docker-compose -f docker-compose.yml -f docker-compose.caddy.yml up -d
```

After that, go to `https://u2vpodcast.tuservidor.com` and you can find a list of the channels. In this page, you can find every channel you added to the configuration file. For example, with **Linux y Tapas**, you  can find,

1. The channel: https://u2vpodcast.tuservidor.com/linux_y_tapas?page=1
2. The feed: https://u2vpodcast.tuservidor.com/linux_y_tapas/feed.xml

### SponsorBlock processing

SponsorBlock is disabled by default. Set `sponsorblock_enabled: true` to opt in; this is also required after upgrading an installation that previously used SponsorBlock. When disabled, u2vpodcast performs no SponsorBlock retrieval, reconciliation, processing, API exposure, manual refresh, player skipping, marker rendering, or processed-feed selection. Existing cached snapshots and derivatives are retained but ignored.

When enabled, u2vpodcast requests `skip` segments for every supported category from the [SponsorBlock](https://github.com/ajayyy/SponsorBlock) [official public service](https://sponsor.ajay.app/) for episodes in the current channel selection window. `sponsorblock_rejected_categories` controls which categories are removed from derived feed audio and skipped by the web player. It defaults to `[sponsor]`; supported values are `sponsor`, `selfpromo`, `interaction`, `intro`, `outro`, `preview`, `music_offtopic`, and `filler`. Duplicates are normalized, unknown values prevent startup, and an explicit empty list rejects nothing. Every available supported category is still shown on enabled web-player timelines, with sponsor and non-sponsor markers visually distinguished.

Configuration is loaded at startup. Rejection metadata exposed to the web app reflects the current configuration immediately after restart, while derived media is reconciled during the next synchronization or authenticated manual refresh. The original-timeline audio is retained as `{yt_id}.mp3` and encoded as 160 kbps CBR so browser seeks map accurately to SponsorBlock timestamps. Existing VBR files must be downloaded again to adopt this encoding. When rejected intervals are available, FFmpeg creates a stream-copy derivative named `{yt_id}.sponsorblock.{processing-hash-prefix}.mp3`; RSS feeds select that derivative and its measured duration only while SponsorBlock is enabled. The web player continues to use the original file and skips only rejected intervals on the original timeline.

An authenticated user can refresh any stored episode, including an older favorite, with `POST /api/1.0/episodes/{yt_id}/sponsorblock/refresh/` while SponsorBlock is enabled. An unchanged processing result reuses the active derivative even when visible, playable segment metadata changes. Empty rejected intervals restore original-media selection. Retrieval, processing, or probing failures preserve the last valid selection, and synchronization continues with later episodes. Cuts use MP3 stream copying, so their boundaries have frame-level rather than sample-level precision.

SponsorBlock segment data is transformed into derived audio cuts for the configured rejected categories. SponsorBlock data is provided under [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/), including its non-commercial restriction. Frontend attribution is intentionally deferred to a follow-up change.

### SQLite & backups

The database runs in WAL journal mode. Backing up the database therefore also requires the companion files `u2vpodcast.db-wal` and `u2vpodcast.db-shm` (or run an explicit SQLite checkpoint/`sqlite3 u2vpodcast.db ".backup backup.db"` before copying only the `.db` file). The pool size is configurable via `db_pool_max_connections` (default 5).

### Contributing

### License

This project is licensed under the MIT license

### Show your support

Leave a ⭐ if you like this project

***
Readme made with 💖 using [README Generator by Dhravya Shah](https://github.com/Dhravya/readme-generator)
