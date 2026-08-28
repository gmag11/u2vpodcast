use sqlx::SqlitePool;
use actix_web::http::StatusCode;
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
use std::{
    collections::HashSet,
    convert::TryFrom,
    path::Path,
    sync::{LazyLock, Mutex},
};
use rand::Rng;
use tokio::fs::create_dir_all;
use tokio::time::sleep;
use std::time::Duration;
use super::super::models::{
    Error,
    Channel,
    Episode,
    PlaylistItem,
    Ytdlp,
    YtVideo,
    audios_dir,
    ytdlp_path,
    cookies_file,
};
use super::sponsorblock::{reconcile_episode, SponsorBlockClient};

// Extra videos requested beyond `max` so exclusions (upcoming/live/future)
// cannot starve the window (scalable-channel-listing).
const MARGIN: usize = 5;

static CHANNEL_SYNCS: LazyLock<Mutex<HashSet<i64>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct ChannelSyncGuard {
    channel_id: i64,
}

impl ChannelSyncGuard {
    fn acquire(channel_id: i64) -> Option<Self> {
        let mut channel_syncs = CHANNEL_SYNCS.lock().unwrap_or_else(|e| e.into_inner());
        if channel_syncs.insert(channel_id) {
            Some(Self { channel_id })
        } else {
            None
        }
    }
}

impl Drop for ChannelSyncGuard {
    fn drop(&mut self) {
        CHANNEL_SYNCS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.channel_id);
    }
}

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
    let _guard = match ChannelSyncGuard::acquire(channel_id) {
        Some(guard) => guard,
        None => {
            info!("Channel {channel_id} sync already in progress; skipping duplicate request");
            return Ok(());
        }
    };
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
    let window_ids = process_channel(pool, &channel, &ytdlp, folder).await?;
    clean_channel(pool, &channel, folder).await?;
    reconcile_sponsorblock_window(
        pool,
        &channel,
        folder,
        &window_ids,
        &SponsorBlockClient::default(),
    )
    .await?;
    // Remove transient/orphan files left by interrupted runs (`.part`,
    // yt-dlp temp files, mp3 without an episode row). Always runs, even when
    // pruning is skipped for an invalid max.
    clean_orphan_files(pool, &channel, folder).await;
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

// True when a file in the channel audio directory is not the finished mp3 of
// a stored episode: yt-dlp/ffmpeg transients (`.part`, `.ytdlp-*`, `.tmp`) and
// any file (mp3 or not) that no episode row references.
fn is_orphan(name: &str, referenced: bool) -> bool {
    if name.ends_with(".part")
        || name.contains(".ytdlp-")
        || name.ends_with(".tmp")
        || name.contains(".tmp.")
    {
        return true;
    }
    !name.ends_with(".mp3") || !referenced
}

// Cleanup for interrupted runs (or container restarts mid-download/conversion):
// removes leftover fragments so they neither accumulate on disk nor ever remain
// half-processed. Runs after the channel's episodes are processed, so a fresh
// download referenced by a stored row is never touched.
async fn clean_orphan_files(pool: &SqlitePool, channel: &Channel, folder: &str) {
    let dir = format!("{folder}/{}", channel.slug);
    let episodes = match Episode::read_episodes_for_channel(pool, channel.id).await {
        Ok(episodes) => episodes,
        Err(error) => {
            error!("Cant load episode files for orphan cleanup: {error}");
            return;
        }
    };
    let mut referenced_files = HashSet::new();
    for episode in episodes {
        referenced_files.insert(format!("{}.mp3", episode.yt_id));
        if let Some(filename) = episode.sponsorblock_processed_filename {
            referenced_files.insert(filename);
        }
    }
    let mut entries = match tokio::fs::read_dir(&dir).await {
        Ok(entries) => entries,
        Err(_) => return, // nothing to clean (e.g. channel never downloaded)
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };
        let referenced = referenced_files.contains(name);
        if is_orphan(name, referenced) {
            info!(
                "Removing orphan file {}/{} (not a stored episode)",
                &channel.slug, name
            );
            let _ = tokio::fs::remove_file(entry.path()).await;
        }
    }
}

fn is_episode_file(name: &str, yt_id: &str) -> bool {
    name == format!("{yt_id}.mp3")
        || name.starts_with(&format!("{yt_id}.mp3."))
        || name.starts_with(&format!("{yt_id}.sponsorblock."))
        || name.starts_with(&format!(".{yt_id}.sponsorblock."))
}

async fn remove_episode_files(channel_dir: &Path, yt_id: &str) {
    let mut entries = match tokio::fs::read_dir(channel_dir).await {
        Ok(entries) => entries,
        Err(_) => return,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();
        if name
            .to_str()
            .map(|name| is_episode_file(name, yt_id))
            .unwrap_or(false)
        {
            let _ = tokio::fs::remove_file(entry.path()).await;
        }
    }
}

// Pure eviction rule over the channel's episodes ordered newest-first (as
// `read_episodes_for_channel` returns them): favorites are skipped entirely
// (never counted, never evicted) and a non-favorite is evicted only once the
// running count of non-favorites exceeds `max` (episode-favorites /
// channel-retention-limit). Because the input is newest-first, the evicted
// rows are exactly the oldest non-favorites.
fn evict_ids(episodes: &[Episode], max: usize) -> Vec<&Episode> {
    let mut evict = Vec::new();
    let mut kept = 0usize;
    for episode in episodes {
        if episode.favorite {
            continue;
        }
        kept += 1;
        if kept > max {
            evict.push(episode);
        }
    }
    evict
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
    for episode in evict_ids(&episodes, max) { // remove
        let channel_dir = Path::new(folder).join(&channel.slug);
        remove_episode_files(&channel_dir, &episode.yt_id).await;
        match Episode::remove(pool, episode.id).await{
            Ok(_) => info!("Removed episode {} and its media", episode.yt_id),
            Err(e) => error!("Cant remove episode {}. {}", episode.yt_id, e),
        }
    }
    Ok(())
}

async fn process_channel(
    pool: &SqlitePool,
    channel: &Channel,
    ytdlp: &Ytdlp,
    folder: &str,
) -> Result<HashSet<String>, Error>{
    info!("Create directory {}/{}", folder, &channel.slug);
    let _ = create_dir_all(format!("{}/{}", folder, channel.slug))
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
    let window_ids = selection
        .window
        .iter()
        .map(|video| video.id.clone())
        .collect::<HashSet<_>>();
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
    Ok(window_ids)
}

fn episodes_in_window<'a>(
    episodes: &'a [Episode],
    window_ids: &HashSet<String>,
) -> Vec<&'a Episode> {
    episodes
        .iter()
        .filter(|episode| window_ids.contains(&episode.yt_id))
        .collect()
}

async fn reconcile_sponsorblock_window(
    pool: &SqlitePool,
    channel: &Channel,
    folder: &str,
    window_ids: &HashSet<String>,
    client: &SponsorBlockClient,
) -> Result<(), Error> {
    let episodes = Episode::read_episodes_for_channel(pool, channel.id).await?;
    let channel_dir = Path::new(folder).join(&channel.slug);
    for episode in episodes_in_window(&episodes, window_ids) {
        if let Err(error) = reconcile_episode(pool, client, episode, &channel_dir).await {
            error!(
                "Cant reconcile SponsorBlock data for episode {}: {}",
                episode.yt_id, error
            );
        }
    }
    Ok(())
}

fn needs_episode_download(episode_exists: bool, original_exists: bool) -> bool {
    !episode_exists || !original_exists
}

fn needs_metadata_probe(episode_exists: bool, video: &YtVideo) -> bool {
    !episode_exists && flat_date(video).is_none()
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
    let filename = format!("{}/{}/{}.mp3",
        folder,
        channel.slug,
        ytvideo.id
    );
    let episode_exists = channel.episode_exists(pool, &ytvideo.id).await;
    if !needs_episode_download(episode_exists, Path::new(&filename).is_file()) {
        info!("El video {} titulado '{}', existe",
            &ytvideo.id,
            &ytvideo.title
        );
        return Ok(());
    }
    if episode_exists {
        info!("The original media for {} is missing; downloading it again", ytvideo.id);
    }
    if needs_metadata_probe(episode_exists, ytvideo) {
        info!("Checking publish date before downloading {}", ytvideo.id);
        let metadata = ytdlp.metadata(&ytvideo.id).await?;
        let published_at = get_published_at(&metadata);
        if published_at < floor {
            info!(
                "Skipping {} published {published_at}: below the {floor} floor",
                ytvideo.id
            );
            return Ok(());
        }
    }
    info!("Downloading video: {:?}", ytvideo);

    // The download run carries the full `yt-dlp` info dict (`--print-json`),
    // so the stored episode is built from authoritative metadata; the flat
    // listing candidate fills any field yt-dlp omitted (scalable-channel
    // -listing).
    let (success, info) = ytdlp.download(&ytvideo.id, &filename).await?;
    if !success{
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
    let delay = rand::thread_rng().gen_range(10..=20);
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
    if episode_exists {
        return Ok(());
    }
    let listen = false;
    let episode = Episode::new(
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
    // Auto-append freshly downloaded episodes to the end of the playlist
    // (auto-playlist-append): reuses the playlist API's "add" semantics
    // (append at end, dedupe via UNIQUE(episode_id)). The append is
    // best-effort: an already-playlisted episode (e.g. a re-downloaded or
    // re-published one) surfaces as a CONFLICT from `add` and is the expected
    // no-op here, and any other failure is logged without aborting the sync
    // run or affecting subsequent downloads.
    match PlaylistItem::add(pool, episode.id).await {
        Ok(_) => info!("Appended {} to the playlist", episode.yt_id),
        Err(e) if e.status_code() == StatusCode::CONFLICT => {
            debug!("Episode {} already in playlist; skipping append", episode.yt_id);
        }
        Err(e) => error!("Cant append episode {} to playlist: {}", episode.yt_id, e),
    }
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
mod channel_sync_guard_tests {
    use super::*;

    #[test]
    fn guard_excludes_only_the_same_channel_and_releases_on_drop() {
        let first = ChannelSyncGuard::acquire(-9_001).expect("first channel sync");
        assert!(ChannelSyncGuard::acquire(-9_001).is_none());

        let other = ChannelSyncGuard::acquire(-9_002).expect("different channel sync");
        drop(other);
        drop(first);

        assert!(ChannelSyncGuard::acquire(-9_001).is_some());
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

#[cfg(test)]
mod episode_download_tests {
    use super::{needs_episode_download, needs_metadata_probe};
    use crate::models::YtVideo;

    fn video(timestamp: Option<i64>, upload_date: &str) -> YtVideo {
        YtVideo {
            id: "video-id".to_string(),
            title: "Video".to_string(),
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

    #[test]
    fn existing_episode_is_downloaded_again_when_original_is_missing() {
        assert!(!needs_episode_download(true, true));
        assert!(needs_episode_download(true, false));
        assert!(needs_episode_download(false, false));
    }

    #[test]
    fn only_new_undated_episodes_need_a_metadata_probe() {
        assert!(needs_metadata_probe(false, &video(None, "")));
        assert!(!needs_metadata_probe(false, &video(Some(1_704_067_200), "")));
        assert!(!needs_metadata_probe(false, &video(None, "20240101")));
        assert!(!needs_metadata_probe(true, &video(None, "")));
    }
}

#[cfg(test)]
mod orphan_tests {
    use super::*;

    #[test]
    fn transients_are_always_orphans() {
        assert!(is_orphan("abc.mp3.part", true));
        assert!(is_orphan("abc.mp3.ytdlp-1a2b3c.tmp", true));
        assert!(is_orphan("abc.part", false));
        assert!(is_orphan(".tmp", false));
    }

    #[test]
    fn referenced_mp3_is_kept_and_others_removed() {
        // Stored episode's mp3 → keep.
        assert!(!is_orphan("abc123.mp3", true));
        // Mp3 without an episode row → orphan.
        assert!(is_orphan("def456.mp3", false));
        // Any non-mp3 final-looking file → orphan (e.g. leftover webm/opus).
        assert!(is_orphan("abc123.webm", true));
        assert!(is_orphan("abc123.opus", true));
    }
}


#[cfg(test)]
mod eviction_tests {
    use super::*;
    use crate::models::{Episode, SponsorBlockCache};
    use sqlx::{
        query,
        migrate::Migrator,
        sqlite::{SqlitePoolOptions, SqliteRow},
        Row,
    };
    use std::path::Path;

    fn mixed_sponsorblock_server(request_count: usize) -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for _ in 0..request_count {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0u8; 4096];
                let read = stream.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..read]);
                let (status, body) = if request.contains("videoID=failed") {
                    ("500 Internal Server Error", "{}")
                } else {
                    ("404 Not Found", "{}")
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        format!("http://{address}")
    }

    async fn memory_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory pool");
        let migrations = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        Migrator::new(migrations)
            .await
            .expect("load migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        pool
    }

    async fn insert_channel(pool: &SqlitePool) -> i64 {
        let now = Utc::now();
        query(
            "INSERT INTO channels (url, title, slug, active, description, image, \
             first, max, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
        )
        .bind("https://example.com/evict")
        .bind("Eviction Test Channel")
        .bind("evict_test_channel")
        .bind(true)
        .bind("")
        .bind("")
        .bind(now)
        .bind(5i64)
        .bind(now)
        .bind(now)
        .map(|row: SqliteRow| row.get::<i64, _>("id"))
        .fetch_one(pool)
        .await
        .expect("insert channel")
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_episode(
        pool: &SqlitePool,
        channel_id: i64,
        yt_id: &str,
        published_at: DateTime<Utc>,
        favorite: bool,
    ) -> i64 {
        let now = Utc::now();
        query(
            "INSERT INTO episodes (channel_id, title, description, yt_id, webpage_url, \
             published_at, duration, image, listen, position_seconds, listened_at, \
             favorite, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING id",
        )
        .bind(channel_id)
        .bind(format!("episode {yt_id}"))
        .bind("")
        .bind(yt_id)
        .bind(format!("https://youtu.be/{yt_id}"))
        .bind(published_at)
        .bind("00:10:00")
        .bind("")
        .bind(false)
        .bind(0i64)
        .bind(Option::<DateTime<Utc>>::None)
        .bind(favorite)
        .bind(now)
        .bind(now)
        .map(|row: SqliteRow| row.get::<i64, _>("id"))
        .fetch_one(pool)
        .await
        .expect("insert episode")
    }

    // Loads the channel's episodes exactly like `clean_channel` does.
    async fn for_channel(pool: &SqlitePool, channel_id: i64) -> Vec<Episode> {
        Episode::read_episodes_for_channel(pool, channel_id)
            .await
            .expect("read episodes")
    }

    fn base_time() -> DateTime<Utc> {
        Utc::now() - chrono::Duration::days(30)
    }

    #[tokio::test]
    async fn favorites_do_not_count_toward_the_limit() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let t0 = base_time();
        // 5 non-favorites (newest 5) + 1 favorite that is the OLDEST stored.
        for i in 0..5 {
            insert_episode(&pool, channel, &format!("nf0{i}"), t0 + chrono::Duration::days(i), false).await;
        }
        insert_episode(&pool, channel, "favold", t0 - chrono::Duration::days(1), true).await;

        let episodes = for_channel(&pool, channel).await;
        let evict = evict_ids(&episodes, 5);
        assert!(evict.is_empty(), "non-favorites sit at max; nothing may be deleted");
    }

    #[tokio::test]
    async fn oldest_favorite_is_never_evicted() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let t0 = base_time();
        // 4 non-favorites + a favorite older than all of them; then a new
        // episode arrives → 5 non-favorites at max: nothing deleted.
        insert_episode(&pool, channel, "favold", t0 - chrono::Duration::days(1), true).await;
        for i in 0..4 {
            insert_episode(&pool, channel, &format!("nf{i}"), t0 + chrono::Duration::days(i), false).await;
        }
        insert_episode(&pool, channel, "nfnew", t0 + chrono::Duration::days(10), false).await;

        let episodes = for_channel(&pool, channel).await;
        let evict = evict_ids(&episodes, 5);
        assert!(evict.is_empty(), "the very old favorite must survive new arrivals");
    }

    #[tokio::test]
    async fn excess_non_favorites_are_evicted_oldest_first() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let t0 = base_time();
        // 6 non-favorites + 1 favorite: only the oldest non-favorite goes.
        let mut oldest = String::new();
        for i in 0..6 {
            let yt_id = format!("nf{i}");
            insert_episode(&pool, channel, &yt_id, t0 + chrono::Duration::days(i), false).await;
            if i == 0 {
                oldest = yt_id;
            }
        }
        insert_episode(&pool, channel, "favx", t0 + chrono::Duration::days(10), true).await;

        let episodes = for_channel(&pool, channel).await;
        let evict = evict_ids(&episodes, 5);
        let ids: Vec<String> = evict.iter().map(|e| e.yt_id.clone()).collect();
        assert_eq!(ids, vec![oldest], "only the oldest non-favorite must be evicted");
    }

    #[tokio::test]
    async fn favorites_survive_repeated_eviction() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let t0 = base_time();
        // max 2: 3 non-favorites + 1 favorite → 1 non-favorite evicted, favorite
        // never in the eviction list.
        insert_episode(&pool, channel, "favkeep", t0 + chrono::Duration::days(5), true).await;
        for i in 0..3 {
            insert_episode(&pool, channel, &format!("nf{i}"), t0 + chrono::Duration::days(i), false).await;
        }

        let episodes = for_channel(&pool, channel).await;
        let evict = evict_ids(&episodes, 2);
        assert_eq!(evict.len(), 1, "exactly the oldest non-favorite must go");
        assert!(
            evict.iter().all(|e| !e.favorite),
            "favorites must never be evicted"
        );
        assert_eq!(evict[0].yt_id, "nf0");
    }

    #[tokio::test]
    async fn sponsorblock_window_includes_recent_episodes_but_not_old_favorites() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let now = Utc::now();
        insert_episode(&pool, channel, "recent", now, false).await;
        insert_episode(&pool, channel, "recent-favorite", now, true).await;
        insert_episode(
            &pool,
            channel,
            "old-favorite",
            now - chrono::Duration::days(30),
            true,
        )
        .await;
        let episodes = for_channel(&pool, channel).await;
        let window_ids = HashSet::from([
            "recent".to_string(),
            "recent-favorite".to_string(),
        ]);

        let selected = episodes_in_window(&episodes, &window_ids)
            .into_iter()
            .map(|episode| episode.yt_id.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(selected, HashSet::from(["recent", "recent-favorite"]));
        assert!(!selected.contains("old-favorite"));
    }

    #[tokio::test]
    async fn retention_and_orphan_cleanup_manage_sponsorblock_files() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        query("UPDATE channels SET max = 1 WHERE id = $1")
            .bind(channel_id)
            .execute(&pool)
            .await
            .unwrap();
        let now = Utc::now();
        let old_id = insert_episode(
            &pool,
            channel_id,
            "old",
            now - chrono::Duration::days(1),
            false,
        )
        .await;
        let new_id = insert_episode(&pool, channel_id, "new", now, false).await;
        SponsorBlockCache::upsert_success(
            &pool,
            old_id,
            &[],
            "old-hash",
            Some("old.sponsorblock.active.mp3"),
            Some(500.0),
        )
        .await
        .unwrap();
        SponsorBlockCache::upsert_success(
            &pool,
            new_id,
            &[],
            "new-hash",
            Some("new.sponsorblock.active.mp3"),
            Some(500.0),
        )
        .await
        .unwrap();
        let root = std::env::temp_dir().join(format!(
            "u2vpodcast-worker-cleanup-{}",
            rand::random::<u64>()
        ));
        let channel_dir = root.join("evict_test_channel");
        std::fs::create_dir_all(&channel_dir).unwrap();
        for name in [
            "old.mp3",
            "old.sponsorblock.active.mp3",
            "old.sponsorblock.stale.mp3",
            ".old.sponsorblock.active.mp3.42.tmp.mp3",
            "new.mp3",
            "new.sponsorblock.active.mp3",
            "new.sponsorblock.stale.mp3",
            ".new.sponsorblock.active.mp3.42.tmp.mp3",
        ] {
            std::fs::write(channel_dir.join(name), b"fixture").unwrap();
        }
        let channel = Channel::read(&pool, channel_id).await.unwrap();
        let root_string = root.to_string_lossy().into_owned();

        clean_channel(&pool, &channel, &root_string).await.unwrap();
        assert!(SponsorBlockCache::read(&pool, old_id).await.unwrap().is_none());
        assert!(std::fs::read_dir(&channel_dir)
            .unwrap()
            .filter_map(Result::ok)
            .all(|entry| !entry.file_name().to_string_lossy().contains("old")));

        clean_orphan_files(&pool, &channel, &root_string).await;
        assert!(channel_dir.join("new.mp3").is_file());
        assert!(channel_dir.join("new.sponsorblock.active.mp3").is_file());
        assert!(!channel_dir.join("new.sponsorblock.stale.mp3").exists());
        assert!(!channel_dir.join(".new.sponsorblock.active.mp3.42.tmp.mp3").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn sponsorblock_failure_does_not_abort_later_window_episodes() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let now = Utc::now();
        let failed_id = insert_episode(&pool, channel_id, "failed", now, false).await;
        let successful_id = insert_episode(
            &pool,
            channel_id,
            "successful",
            now - chrono::Duration::seconds(1),
            false,
        )
        .await;
        let channel = Channel::read(&pool, channel_id).await.unwrap();
        let root = std::env::temp_dir().join(format!(
            "u2vpodcast-worker-reconcile-{}",
            rand::random::<u64>()
        ));
        std::fs::create_dir_all(root.join(&channel.slug)).unwrap();
        let client = SponsorBlockClient::new(
            &mixed_sponsorblock_server(2),
            Duration::from_secs(1),
        );
        let window_ids = HashSet::from(["failed".to_string(), "successful".to_string()]);

        reconcile_sponsorblock_window(
            &pool,
            &channel,
            &root.to_string_lossy(),
            &window_ids,
            &client,
        )
        .await
        .expect("mixed reconciliation completes");

        assert!(SponsorBlockCache::read(&pool, failed_id).await.unwrap().is_none());
        let successful = SponsorBlockCache::read(&pool, successful_id)
            .await
            .unwrap()
            .expect("later episode persisted");
        assert!(successful.segments.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
