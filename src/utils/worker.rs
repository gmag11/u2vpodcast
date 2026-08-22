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

// Extra videos requested beyond `max` so exclusions (upcoming/live/future)
// cannot starve the window (scalable-channel-listing).
const MARGIN: usize = 5;

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
    info!("Syncing channel: {}", channel);
    // The sync window targets the `max` most recent videos (newest-first via
    // the channel `/videos` tab order). A small margin is requested so that
    // exclusions (upcoming/live/future) cannot starve the window
    // (scalable-channel-listing).
    let max: usize = usize::try_from(channel.max.max(1)).unwrap_or(1);
    let wanted = max.saturating_add(MARGIN);
    info!("Listing up to {wanted} recent videos (window {max}) for {}", channel);
    let candidates = ytdlp.list_videos(&channel.url, wanted).await?;
    let selection = select_window(candidates, max, channel.first, Utc::now());
    info!(
        "Candidate window: {} videos ({} excluded as upcoming/live/future, floor break: {})",
        selection.window.len(),
        selection.excluded,
        selection.stopped_at_floor
    );
    for (index, ytvideo) in selection.window.iter().enumerate(){
        info!("Processing {}/{}: {}", index + 1, selection.window.len(), ytvideo.title);
        match process_episode(pool, channel, ytvideo, ytdlp, folder, channel.first).await{
            Ok(_) => {},
            Err(e) => error!("Cant process episode: {e}"),
        }
    }
    Ok(())
}

async fn process_episode(
    pool: &SqlitePool,
    channel: &Channel,
    ytvideo: &YtVideo,
    ytdlp: &Ytdlp,
    folder: &str,
    floor: DateTime<Utc>,
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
    // Authoritative re-check against the `first` floor: a candidate that
    // carried no usable date (and so survived selection) may turn out to be
    // older than the floor once the real metadata is known — drop it and its
    // downloaded file instead of storing an episode below the floor.
    if published_at < floor {
        info!(
            "Discarding {} published {published_at}: below the {floor} floor",
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

// Best parseable date of a flat candidate, without a fallback: `None` when the
// entry carries no usable date (timestamp, upload_date or release_date).
fn flat_date(video: &YtVideo) -> Option<DateTime<Utc>> {
    if let Some(timestamp) = video.timestamp {
        if let Some(dt) = TimeZone::timestamp_opt(&Utc, timestamp, 0).single() {
            return Some(dt);
        }
    }
    for candidate in [&video.upload_date, &video.release_date] {
        if let Ok(naive_date) = NaiveDate::parse_from_str(candidate, "%Y%m%d") {
            if let Some(naive_datetime) = naive_date.and_hms_opt(0, 0, 0) {
                return Some(TimeZone::from_utc_datetime(&Utc, &naive_datetime));
            }
        }
    }
    None
}

/// Result of selecting the count-window from the flat listing.
pub(crate) struct WindowSelection {
    pub(crate) window: Vec<YtVideo>,
    pub(crate) excluded: usize,
    pub(crate) stopped_at_floor: bool,
}

// Count-window selection (scalable-channel-listing): candidates are taken in
// listing order (the `/videos` tab is newest-first). `is_upcoming`/`is_live`
// and future-dated entries (beyond a 1h clock-skew tolerance) are excluded;
// the scan stops at the first entry older than the `first` floor (the rest are
// older still). Entries without a date keep their listing position - the floor
// is enforced at download with the authoritative date.
pub(crate) fn select_window(
    candidates: Vec<YtVideo>,
    max: usize,
    floor: DateTime<Utc>,
    now: DateTime<Utc>,
) -> WindowSelection {
    let future_limit = now + chrono::Duration::hours(1);
    let mut window = Vec::with_capacity(max);
    let mut excluded = 0usize;
    let mut stopped_at_floor = false;
    for video in candidates {
        if window.len() >= max {
            break;
        }
        match video.live_status.as_str() {
            "is_upcoming" | "is_live" => {
                excluded += 1;
                continue;
            }
            _ => {}
        }
        if let Some(date) = flat_date(&video) {
            if date > future_limit {
                excluded += 1;
                continue;
            }
            if date < floor {
                stopped_at_floor = true;
                break;
            }
        }
        window.push(video);
    }
    WindowSelection {
        window,
        excluded,
        stopped_at_floor,
    }
}

#[cfg(test)]
mod selection_tests {
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
            release_date: String::new(),
            live_status: String::new(),
        }
    }

    fn day_seconds(days: i64) -> i64 {
        1_704_067_200 + days * 86_400 // 2024-01-01 UTC base
    }

    #[test]
    fn window_is_the_max_most_recent_in_listing_order() {
        // Newest-first, exactly as the `/videos` tab presents them.
        let candidates = (0..10)
            .map(|i| video(&format!("v{i}"), Some(day_seconds(10 - i as i64)), ""))
            .collect();
        let floor = TimeZone::timestamp_opt(&Utc, 1_670_000_000, 0).unwrap();
        let now = TimeZone::timestamp_opt(&Utc, day_seconds(20), 0).unwrap();
        let selection = select_window(candidates, 3, floor, now);
        assert_eq!(
            selection.window.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(),
            vec!["v0", "v1", "v2"]
        );
        assert_eq!(selection.excluded, 0);
        assert!(!selection.stopped_at_floor);
    }

    #[test]
    fn upcoming_live_and_future_dated_are_excluded() {
        let mut upcoming = video("upcoming", None, "");
        upcoming.live_status = "is_upcoming".to_string();
        let mut live = video("live", None, "");
        live.live_status = "is_live".to_string();
        let future = video("future", Some(day_seconds(25)), ""); // beyond now+1h
        let now = TimeZone::timestamp_opt(&Utc, day_seconds(20), 0).unwrap();

        // Listing order: upcoming, live, future first; then real dated ones.
        let candidates = vec![
            upcoming,
            live,
            future,
            video("recent", Some(day_seconds(18)), ""),
            video("older", Some(day_seconds(17)), ""),
        ];
        let floor = TimeZone::timestamp_opt(&Utc, 1_670_000_000, 0).unwrap();
        let selection = select_window(candidates, 10, floor, now);
        assert_eq!(
            selection.window.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(),
            vec!["recent", "older"]
        );
        assert_eq!(selection.excluded, 3);
    }

    #[test]
    fn floor_stops_the_scan_at_an_older_entry() {
        let floor = TimeZone::timestamp_opt(&Utc, day_seconds(10), 0).unwrap();
        let now = TimeZone::timestamp_opt(&Utc, day_seconds(20), 0).unwrap();
        let candidates = vec![
            video("new1", Some(day_seconds(13)), ""),
            video("new2", Some(day_seconds(11)), ""),
            video("below_floor", Some(day_seconds(9)), ""), // < floor → stop
            video("much_older", Some(day_seconds(2)), ""),
        ];
        let selection = select_window(candidates, 10, floor, now);
        assert_eq!(
            selection.window.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(),
            vec!["new1", "new2"]
        );
        assert!(selection.stopped_at_floor);
    }

    #[test]
    fn undated_entries_keep_their_listing_position() {
        let floor = TimeZone::timestamp_opt(&Utc, 1_670_000_000, 0).unwrap();
        let now = TimeZone::timestamp_opt(&Utc, day_seconds(20), 0).unwrap();
        let candidates = vec![
            video("dated1", Some(day_seconds(18)), ""),
            video("undated", None, ""), // keeps position 2 in listing order
            video("dated2", Some(day_seconds(16)), ""),
        ];
        let selection = select_window(candidates, 3, floor, now);
        assert_eq!(
            selection.window.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(),
            vec!["dated1", "undated", "dated2"]
        );
    }

    #[test]
    fn raising_max_includes_older_missing_episodes() {
        // 35 entries, newest (day 35) -> oldest (day 1), all within floor.
        let mut candidates = Vec::new();
        for i in 0..35 {
            candidates.push(video(&format!("v{i}"), Some(day_seconds(35 - i as i64)), ""));
        }
        let floor = TimeZone::timestamp_opt(&Utc, 1_600_000_000, 0).unwrap();
        let now = TimeZone::timestamp_opt(&Utc, day_seconds(40), 0).unwrap();

        // max=20 → the 20 newest.
        let twenty = select_window(candidates.clone(), 20, floor, now);
        assert_eq!(twenty.window.len(), 20);
        assert_eq!(twenty.window[0].id, "v0");
        assert_eq!(twenty.window[19].id, "v19");

        // max=30 → the 21st-30th enter the window (the older missing ones).
        let thirty = select_window(candidates, 30, floor, now);
        assert_eq!(thirty.window.len(), 30);
        assert_eq!(thirty.window[20].id, "v20");
        assert_eq!(thirty.window[29].id, "v29");
    }
}

