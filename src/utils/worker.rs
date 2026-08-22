use sqlx::SqlitePool;
use tracing::{
    info,
    debug,
    error,
};
use chrono::{
    Utc,
    DateTime,
    TimeZone,
    naive::{
        NaiveDate,
    },
};
use std::convert::TryFrom;
use rand::Rng;
use tokio::fs::create_dir_all;
use tokio::time::sleep;
use std::time::Duration;
use super::super::models::{
    Error,
    Channel,
    Episode,
    Ytdlp,
    YtVideo,
    audios_dir,
    ytdlp_path,
    cookies_file,
};

pub async fn do_the_work(pool: &SqlitePool) -> Result<(), Error>{
    let channels = Channel::read_active(pool).await?;
    for channel in channels.as_slice(){
        info!("Processing: {}", channel.url);
        // Run each channel in its own isolated task so that a panic while
        // processing one channel (e.g. yt-dlp output, filesystem) can never
        // unwind through here and kill the scheduled worker loop.
        let pool = pool.clone();
        let url_for_task = channel.url.clone();
        let channel_id = channel.id;
        match tokio::spawn(async move {
            if let Err(e) = update_channel(&pool, channel_id).await {
                error!("Cant process channel: {url_for_task}. Error: {e}");
            }
        }).await {
            Ok(()) => {},
            Err(e) => error!("Task panicked while processing channel: {}. Error: {e}", channel.url),
        }
    }
    Ok(())
}

pub async fn update_channel(pool: &SqlitePool, channel_id: i64) -> Result<(), Error>{
    let (ok, message) = match update_channel_inner(pool, channel_id).await {
        Ok(()) => (true, None),
        Err(e) => (false, Some(error_message(e))),
    };
    let _ = Channel::set_sync_status(pool, channel_id, ok, message.clone()).await;
    if ok {
        Ok(())
    } else {
        Err(Error::default(&message.unwrap_or_default()))
    }
}

fn error_message(e: Error) -> String {
    e.to_string()
}

async fn update_channel_inner(pool: &SqlitePool, channel_id: i64) -> Result<(), Error>{
    let channel = Channel::read(pool, channel_id).await?;
    let ytdlp = Ytdlp::new(ytdlp_path(), cookies_file());
    let folder = audios_dir();
    process_channel(pool, &channel, &ytdlp, folder).await?;
    clean_channel(pool, &channel, folder).await?;
    // Refresh the cached cover as part of the sync so it stays current between
    // cycles. Skipped for inactive channels per the `active` flag semantics:
    // the scheduled worker only picks active channels, and a forced sync of an
    // inactive one must not start image traffic. Best-effort: a failing image
    // refresh is logged and does not fail the channel sync.
    if channel.active {
        if let Err(e) = Channel::refresh_cached_image(pool, &channel).await {
            error!("Cant refresh cached image for channel {}: {}", channel.id, e);
        }
    }
    info!("Channel {} updated", &channel.id);
    Ok(())
}

async fn clean_channel(pool: &SqlitePool, channel: &Channel, folder: &str) -> Result<(), Error>{
    // Defense in depth: a stored `max` below 1 (e.g. the historical -1 default)
    // must never trigger mass deletion and must not fail the sync. We keep all
    // episodes unchanged until the operator sets a valid retention limit.
    let max = match usize::try_from(channel.max) {
        Ok(m) if m >= 1 => m,
        _ => {
            info!(
                "Skipping prune for channel {}: max={} is invalid; keeping all episodes",
                channel.id, channel.max
            );
            return Ok(());
        }
    };
    let episodes = Episode::read_episodes_for_channel(pool, channel.id).await?;
    for (index, episode) in episodes.iter().enumerate(){
        if index >= max { // remove
            let filename = format!("{}/{}/{}.mp3", folder, &channel.slug, episode.yt_id);
            info!("Deleting file {filename}");
            let exists = tokio::fs::metadata(&filename)
                .await
                .map(|f| f.is_file())
                .unwrap_or(false);
            let removed = tokio::fs::remove_file(&filename)
                .await
                .map(|_| true)
                .unwrap_or(false);
            if !exists || removed {
                match Episode::remove(pool, episode.id).await{
                    Ok(_) => info!("Removed {}", &filename),
                    Err(e) => error!("Cant remove {}. {}", &filename, e),
                }
            }
        }
    }
    Ok(())
}

async fn process_channel(
    pool: &SqlitePool,
    channel: &Channel,
    ytdlp: &Ytdlp,
    folder: &str,
) -> Result<(), Error>{
    info!("Create directory {}/{}", folder, &channel.slug);
    let _ = create_dir_all(format!("{}/{}", folder, &channel.slug))
        .await;
    info!("Getting new videos for channel: {}", channel);
    let first = channel.first;
    let last = if channel.number_of_episodes(pool).await > 0 {
        let last = channel.get_max_date(pool).await;
        if last < first{
            first
        }else{
            last
        }
    }else{
        first
    };
    info!("Last video: {}", &last);
    let mut ytvideos = ytdlp.get_latest(&channel.url, last).await?;
    // Backstop: `get_latest` now runs flat, and `--dateafter` may not apply to
    // flat entries in every extractor version. Filter by the same `last`
    // boundary here so out-of-window videos never trigger a per-video
    // connection (scalable-channel-listing).
    let before = ytvideos.len();
    filter_by_window(&mut ytvideos, last);
    if ytvideos.len() != before {
        info!(
            "Backstop date filter dropped {} out-of-window candidates",
            before - ytvideos.len()
        );
    }
    info!("Getting {} videos", ytvideos.len());
    for ytvideo in ytvideos{
        info!("Processing: {}", &ytvideo.title);
        match process_episode(pool, channel, &ytvideo, ytdlp, folder, last).await{
            Ok(_) => {},
            Err(e) => error!("Cant process episode: {e}"),
        }
    }
    //TODO: Delete older episodes
    Ok(())
}

async fn process_episode(
    pool: &SqlitePool,
    channel: &Channel,
    ytvideo: &YtVideo,
    ytdlp: &Ytdlp,
    folder: &str,
    window_start: DateTime<Utc>,
) -> Result<(), Error>{
    info!("Start processing episode {}", ytvideo.title);
    if channel.episode_exists(pool, &ytvideo.id).await{
        info!("El video {} titulado '{}', existe",
            &ytvideo.id,
            &ytvideo.title
        );
        return Ok(());
    }
    info!("Downloading video: {:?}", ytvideo);
    let filename = format!("{}/{}/{}.mp3",
        folder,
        channel.slug,
        &ytvideo.id
    );

    // The download run carries the full `yt-dlp` info dict (`--print-json`),
    // so the stored episode is built from authoritative metadata; the flat
    // listing candidate fills any field yt-dlp omitted (scalable-channel
    // -listing).
    let (status, info) = ytdlp.download(&ytvideo.id, &filename).await?;
    if !status.success(){
        Err(Error::default(&format!("Cant download {filename}")))?
    }
    let published_at = get_published_at(&info);
    // Authoritative re-check against the date window: a flat candidate that
    // carried no date (and so survived the backstop) may turn out to be out of
    // window once the real metadata is known — drop it and its downloaded
    // file instead of storing an episode outside the window.
    if published_at < window_start {
        info!(
            "Discarding {} published {published_at}: before the {window_start} window",
            ytvideo.id
        );
        let _ = tokio::fs::remove_file(&filename).await;
        return Ok(());
    }
    let delay = rand::thread_rng().gen_range(20..=40);
    info!("Pausing {delay} seconds before next download");
    sleep(Duration::from_secs(delay)).await;
    let title = if info.title.is_empty() { &ytvideo.title } else { &info.title };
    let description = if info.description.is_empty() { &ytvideo.description } else { &info.description };
    let yt_id = if info.id.is_empty() { &ytvideo.id } else { &info.id };
    let webpage_url = if info.webpage_url.is_empty() { &ytvideo.webpage_url } else { &info.webpage_url };
    let duration = if info.duration_string.is_empty() { &ytvideo.duration_string } else { &info.duration_string };
    let image = if info.thumbnail.is_empty() { &ytvideo.thumbnail } else { &info.thumbnail };
    info!("{}", &info.upload_date);
    let _ = filetime::set_file_mtime(
        &filename,
        filetime::FileTime::from_unix_time(
            published_at.timestamp(), 0)
    );
    let listen = false;
    let _ = Episode::new(
        pool,
        channel.id,
        title,
        description,
        yt_id,
        webpage_url,
        &published_at,
        duration,
        image,
        listen
    ).await?;
    Ok(())
}

fn get_published_at(ytvideo: &YtVideo) -> DateTime<Utc>{
    // Prefer the precise publish timestamp when available.
    if let Some(timestamp) = ytvideo.timestamp {
        if let Some(dt) = TimeZone::timestamp_opt(&Utc, timestamp, 0).single() {
            return dt;
        }
    }
    let format = "%Y%m%d";
    if let Ok(naive_date) = NaiveDate::parse_from_str(&ytvideo.upload_date, format) {
        // Add some default time to convert it into a NaiveDateTime
        if let Some(naive_datetime) = naive_date.and_hms_opt(0, 0, 0) {
            // Add a timezone to the object to convert it into a DateTime<UTC>
            return TimeZone::from_utc_datetime(&Utc, &naive_datetime);
        }
    }
    // Fallback so a malformed/youtube edge-case date never panics the worker,
    // and flat-listing entries that omit dates do not spam the ERROR log (the
    // candidate is conservatively kept; the authoritative date comes from the
    // per-video detail/download metadata - scalable-channel-listing).
    debug!("Cant parse publish date from {:?}, using now()", &ytvideo.upload_date);
    Utc::now()
}

// Backstop date filter applied to flat-listing candidates (scalable-channel
// -listing): keeps videos whose parseable date is on/after the window
// boundary. Candidates with no parseable date are conservatively kept
// (`get_published_at` falls back to now()).
pub(crate) fn filter_by_window(videos: &mut Vec<YtVideo>, since: DateTime<Utc>) {
    videos.retain(|video| get_published_at(video) >= since);
}

#[cfg(test)]
mod backstop_tests {
    use super::*;

    fn video(id: &str, timestamp: Option<i64>, upload_date: &str) -> YtVideo {
        YtVideo {
            id: id.to_string(),
            title: format!("Title {id}"),
            description: String::new(),
            thumbnail: String::new(),
            original_url: String::new(),
            webpage_url: String::new(),
            upload_date: upload_date.to_string(),
            timestamp,
            duration_string: String::new(),
        }
    }

    #[test]
    fn keeps_only_in_window_candidates() {
        let since = TimeZone::timestamp_opt(&Utc, 1_704_067_200, 0).unwrap(); // 2024-01-01 UTC
        let mut videos = vec![
            video("older", Some(since.timestamp() - 86_400), "20231231"),
            video("on_edge", Some(since.timestamp()), "20240101"),
            video("newer", Some(since.timestamp() + 86_400), "20240102"),
            video("no_date", None, ""), // conservative keep
        ];
        filter_by_window(&mut videos, since);
        let ids: Vec<&str> = videos.iter().map(|v| v.id.as_str()).collect();
        assert_eq!(ids, vec!["on_edge", "newer", "no_date"]);
    }
}

