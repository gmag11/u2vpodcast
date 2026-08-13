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

If only one of `admin_username` and `admin_password` is set (or both are missing), both are **ignored**: the `users` table is left untouched on startup and the service authenticates against the admin account already stored in the database. To adopt this mode, log in once with the seeded credentials so the user row exists, then remove both keys from `config.yml` and restart.

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

### Contributing

### License

This project is licensed under the MIT license

### Show your support

Leave a ⭐ if you like this project

***
Readme made with 💖 using [README Generator by Dhravya Shah](https://github.com/Dhravya/readme-generator)
