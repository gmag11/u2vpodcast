use crate::models::{Episode, Error, SponsorBlockCache, SponsorBlockSegment};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::Command,
    time::Duration,
};
use ureq::{Agent, Error as UreqError};

const SPONSORBLOCK_BASE_URL: &str = "https://sponsor.ajay.app";
const SPONSORBLOCK_TIMEOUT: Duration = Duration::from_secs(15);
const RESPONSE_MAX_BYTES: u64 = 1024 * 1024;
const PROCESSING_FORMAT_VERSION: u32 = 1;
const SELECTED_CATEGORIES: [&str; 1] = ["sponsor"];

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SponsorBlockApiSegment {
    pub segment: [f64; 2],
    pub category: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
}

#[derive(Debug, Clone)]
pub struct SponsorBlockClient {
    base_url: String,
    timeout: Duration,
}

impl Default for SponsorBlockClient {
    fn default() -> Self {
        Self::new(SPONSORBLOCK_BASE_URL, SPONSORBLOCK_TIMEOUT)
    }
}

impl SponsorBlockClient {
    pub fn new(base_url: &str, timeout: Duration) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout,
        }
    }

    pub async fn fetch(&self, video_id: &str) -> Result<Vec<SponsorBlockApiSegment>, String> {
        let base_url = self.base_url.clone();
        let timeout = self.timeout;
        let video_id = video_id.to_string();
        actix_web::rt::task::spawn_blocking(move || {
            fetch_blocking(&base_url, timeout, &video_id)
        })
        .await
        .map_err(|error| format!("SponsorBlock task failed: {error}"))?
    }
}

fn fetch_blocking(
    base_url: &str,
    timeout: Duration,
    video_id: &str,
) -> Result<Vec<SponsorBlockApiSegment>, String> {
    let agent: Agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build()
        .into();
    let url = format!("{base_url}/api/skipSegments");
    let response = match agent
        .get(&url)
        .query("videoID", video_id)
        .query("categories", r#"["sponsor"]"#)
        .query("actionTypes", r#"["skip"]"#)
        .header("User-Agent", concat!("u2vpodcast/", env!("CARGO_PKG_VERSION")))
        .call()
    {
        Ok(response) => response,
        Err(UreqError::StatusCode(404)) => return Ok(Vec::new()),
        Err(error) => return Err(format!("SponsorBlock request failed: {error}")),
    };
    let mut response = response;
    let body = response
        .body_mut()
        .with_config()
        .limit(RESPONSE_MAX_BYTES)
        .read_to_string()
        .map_err(|error| format!("read SponsorBlock response: {error}"))?;
    serde_json::from_str(&body)
        .map_err(|error| format!("parse SponsorBlock response: {error}"))
}

pub fn normalize_segments(
    segments: &[SponsorBlockApiSegment],
    original_duration: Option<f64>,
) -> Vec<SponsorBlockSegment> {
    let duration = original_duration.filter(|duration| duration.is_finite() && *duration >= 0.0);
    let mut normalized = segments
        .iter()
        .filter(|segment| segment.category == "sponsor" && segment.action_type == "skip")
        .filter_map(|segment| {
            let [mut start, mut end] = segment.segment;
            if !start.is_finite() || !end.is_finite() {
                return None;
            }
            start = start.max(0.0);
            end = end.max(0.0);
            if let Some(duration) = duration {
                start = start.min(duration);
                end = end.min(duration);
            }
            (end > start).then_some(SponsorBlockSegment { start, end })
        })
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| left.start.total_cmp(&right.start));

    let mut merged: Vec<SponsorBlockSegment> = Vec::with_capacity(normalized.len());
    for segment in normalized {
        if let Some(previous) = merged.last_mut() {
            if segment.start <= previous.end {
                previous.end = previous.end.max(segment.end);
                continue;
            }
        }
        merged.push(segment);
    }
    merged
}

#[derive(Serialize)]
struct CanonicalSnapshot<'a> {
    processing_format_version: u32,
    categories: &'a [&'a str],
    segments: &'a [SponsorBlockSegment],
}

pub fn snapshot_hash(segments: &[SponsorBlockSegment]) -> String {
    snapshot_hash_for(segments, &SELECTED_CATEGORIES, PROCESSING_FORMAT_VERSION)
}

fn snapshot_hash_for(
    segments: &[SponsorBlockSegment],
    categories: &[&str],
    processing_format_version: u32,
) -> String {
    let payload = serde_json::to_vec(&CanonicalSnapshot {
        processing_format_version,
        categories,
        segments,
    })
    .expect("canonical SponsorBlock snapshot is serializable");
    let digest = Sha256::digest(payload);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn retained_intervals(
    sponsor_segments: &[SponsorBlockSegment],
    original_duration: f64,
) -> Vec<SponsorBlockSegment> {
    if !original_duration.is_finite() || original_duration <= 0.0 {
        return Vec::new();
    }
    let mut retained = Vec::new();
    let mut cursor = 0.0;
    for segment in sponsor_segments {
        let start = segment.start.clamp(0.0, original_duration);
        let end = segment.end.clamp(0.0, original_duration);
        if start > cursor {
            retained.push(SponsorBlockSegment { start: cursor, end: start });
        }
        cursor = cursor.max(end);
    }
    if cursor < original_duration {
        retained.push(SponsorBlockSegment {
            start: cursor,
            end: original_duration,
        });
    }
    retained
}

pub fn ffconcat_manifest(original: &Path, retained: &[SponsorBlockSegment]) -> String {
    let escaped_path = original
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "'\\''");
    let mut manifest = "ffconcat version 1.0\n".to_string();
    for interval in retained {
        manifest.push_str(&format!(
            "file '{escaped_path}'\ninpoint {:.6}\noutpoint {:.6}\n",
            interval.start, interval.end
        ));
    }
    manifest
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedMedia {
    pub filename: String,
    pub duration: f64,
}

pub async fn generate_processed_mp3(
    original: &Path,
    sponsor_segments: &[SponsorBlockSegment],
    original_duration: f64,
    hash: &str,
) -> Result<ProcessedMedia, String> {
    let original = original.to_path_buf();
    let sponsor_segments = sponsor_segments.to_vec();
    let hash = hash.to_string();
    actix_web::rt::task::spawn_blocking(move || {
        generate_processed_mp3_blocking(&original, &sponsor_segments, original_duration, &hash)
    })
    .await
    .map_err(|error| format!("SponsorBlock media task failed: {error}"))?
}

fn generate_processed_mp3_blocking(
    original: &Path,
    sponsor_segments: &[SponsorBlockSegment],
    original_duration: f64,
    hash: &str,
) -> Result<ProcessedMedia, String> {
    let retained = retained_intervals(sponsor_segments, original_duration);
    if retained.is_empty() {
        return Err("SponsorBlock segments leave no original audio to retain".to_string());
    }
    let parent = original.parent().ok_or_else(|| "original MP3 has no parent".to_string())?;
    let stem = original
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| "original MP3 has no valid file stem".to_string())?;
    let hash_prefix = hash.get(..16).ok_or_else(|| "SponsorBlock hash is too short".to_string())?;
    let filename = format!("{stem}.sponsorblock.{hash_prefix}.mp3");
    let destination = parent.join(&filename);
    let nonce = rand::random::<u64>();
    let manifest_path = parent.join(format!(".{filename}.{nonce}.ffconcat.tmp"));
    let temporary_path = parent.join(format!(".{filename}.{nonce}.tmp.mp3"));

    let result = (|| {
        let manifest_source = original
            .file_name()
            .map(Path::new)
            .ok_or_else(|| "original MP3 has no filename".to_string())?;
        fs::write(&manifest_path, ffconcat_manifest(manifest_source, &retained))
            .map_err(|error| format!("write ffconcat manifest: {error}"))?;
        let output = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i"])
            .arg(&manifest_path)
            .args(["-map", "0:a:0", "-c:a", "copy", "-y"])
            .arg(&temporary_path)
            .output()
            .map_err(|error| format!("start FFmpeg: {error}"))?;
        if !output.status.success() {
            return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
        }
        if fs::metadata(&temporary_path).map(|metadata| metadata.len()).unwrap_or(0) == 0 {
            return Err("FFmpeg produced an empty MP3".to_string());
        }
        let duration = probe_duration(&temporary_path)?;
        fs::rename(&temporary_path, &destination)
            .map_err(|error| format!("publish processed MP3: {error}"))?;
        Ok(ProcessedMedia { filename, duration })
    })();

    let _ = fs::remove_file(&manifest_path);
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn probe_duration(path: &Path) -> Result<f64, String> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(path)
        .output()
        .map_err(|error| format!("start ffprobe: {error}"))?;
    if !output.status.success() {
        return Err(format!("ffprobe failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    let duration = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|error| format!("parse ffprobe duration: {error}"))?;
    if !duration.is_finite() || duration <= 0.0 {
        return Err("ffprobe returned an invalid duration".to_string());
    }
    Ok(duration)
}

pub fn parse_duration_seconds(duration: &str) -> Option<f64> {
    let mut seconds = 0.0;
    let parts = duration.trim().split(':').collect::<Vec<_>>();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    for part in parts {
        seconds = seconds * 60.0 + part.parse::<f64>().ok()?;
    }
    (seconds.is_finite() && seconds > 0.0).then_some(seconds)
}

pub async fn reconcile_episode(
    pool: &sqlx::SqlitePool,
    client: &SponsorBlockClient,
    episode: &Episode,
    channel_dir: &Path,
) -> Result<SponsorBlockCache, Error> {
    let original_duration = parse_duration_seconds(&episode.duration)
        .ok_or_else(|| Error::default("episode has an invalid duration"))?;
    let response = match client.fetch(&episode.yt_id).await {
        Ok(response) => response,
        Err(message) => {
            let _ = SponsorBlockCache::record_failure(pool, episode.id, &message).await;
            return Err(Error::default(&message));
        }
    };
    let segments = normalize_segments(&response, Some(original_duration));
    let hash = snapshot_hash(&segments);
    let previous = SponsorBlockCache::read(pool, episode.id).await?;

    if let Some(active) = previous.as_ref().filter(|active| active.snapshot_hash == hash) {
        return SponsorBlockCache::upsert_success(
            pool,
            episode.id,
            &segments,
            &hash,
            active.processed_filename.as_deref(),
            active.processed_duration,
        )
        .await;
    }

    if segments.is_empty() {
        let current = SponsorBlockCache::upsert_success(
            pool,
            episode.id,
            &segments,
            &hash,
            None,
            None,
        )
        .await?;
        if let Some(filename) = previous.and_then(|active| active.processed_filename) {
            let _ = fs::remove_file(channel_dir.join(filename));
        }
        return Ok(current);
    }

    let original = channel_dir.join(format!("{}.mp3", episode.yt_id));
    let processed = match generate_processed_mp3(&original, &segments, original_duration, &hash).await {
        Ok(processed) => processed,
        Err(message) => {
            let _ = SponsorBlockCache::record_failure(pool, episode.id, &message).await;
            return Err(Error::default(&message));
        }
    };
    let current = match SponsorBlockCache::upsert_success(
        pool,
        episode.id,
        &segments,
        &hash,
        Some(&processed.filename),
        Some(processed.duration),
    )
    .await
    {
        Ok(current) => current,
        Err(error) => {
            let _ = fs::remove_file(channel_dir.join(&processed.filename));
            return Err(error);
        }
    };
    if let Some(filename) = previous
        .and_then(|active| active.processed_filename)
        .filter(|filename| filename != &processed.filename)
    {
        let _ = fs::remove_file(channel_dir.join(filename));
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        path::PathBuf,
        thread,
    };
    use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};

    fn media_fixture_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "u2vpodcast-sponsorblock-media-{}",
            rand::random::<u64>()
        ));
        fs::create_dir_all(&dir).expect("create media fixture directory");
        dir
    }

    async fn reconciliation_fixture() -> (sqlx::SqlitePool, Episode, PathBuf) {
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
        let now = chrono::Utc::now();
        let channel_id: i64 = sqlx::query_scalar(
            "INSERT INTO channels (url, title, slug, active, description, image, first, max, created_at, updated_at) \
             VALUES ('https://example.com', 'Channel', 'channel', TRUE, '', '', $1, 5, $1, $1) RETURNING id",
        )
        .bind(now)
        .fetch_one(&pool)
        .await
        .unwrap();
        let episode_id: i64 = sqlx::query_scalar(
            "INSERT INTO episodes (channel_id, title, yt_id, webpage_url, published_at, duration, created_at, updated_at) \
             VALUES ($1, 'Episode', 'video-id', 'https://example.com/video', $2, '00:00:03', $2, $2) RETURNING id",
        )
        .bind(channel_id)
        .bind(now)
        .fetch_one(&pool)
        .await
        .unwrap();
        let episode = Episode::read(&pool, episode_id).await.unwrap();
        (pool, episode, media_fixture_dir())
    }

    fn server(status: &str, body: &str, delay: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
        let address = listener.local_addr().expect("fixture address");
        let status = status.to_string();
        let body = body.to_string();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept fixture request");
            let mut request = [0u8; 4096];
            let read = stream.read(&mut request).expect("read fixture request");
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.contains("videoID=video-id"));
            assert!(request.contains("categories=%5B%22sponsor%22%5D"));
            assert!(request.contains("actionTypes=%5B%22skip%22%5D"));
            thread::sleep(delay);
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
        });
        format!("http://{address}")
    }

    fn raw(start: f64, end: f64) -> SponsorBlockApiSegment {
        SponsorBlockApiSegment {
            segment: [start, end],
            category: "sponsor".to_string(),
            action_type: "skip".to_string(),
        }
    }

    #[test]
    fn normalizes_filters_clamps_sorts_and_merges_segments() {
        let mut irrelevant_category = raw(30.0, 40.0);
        irrelevant_category.category = "intro".to_string();
        let mut irrelevant_action = raw(40.0, 50.0);
        irrelevant_action.action_type = "mute".to_string();
        let input = [
            raw(20.0, 30.0),
            raw(-5.0, 5.0),
            raw(4.0, 10.0),
            raw(10.0, 12.0),
            raw(80.0, 120.0),
            raw(50.0, 50.0),
            raw(f64::NAN, 60.0),
            irrelevant_category,
            irrelevant_action,
        ];
        assert_eq!(
            normalize_segments(&input, Some(100.0)),
            [
                SponsorBlockSegment { start: 0.0, end: 12.0 },
                SponsorBlockSegment { start: 20.0, end: 30.0 },
                SponsorBlockSegment { start: 80.0, end: 100.0 },
            ]
        );
    }

    #[test]
    fn normalization_handles_empty_unknown_and_invalid_duration() {
        assert!(normalize_segments(&[], Some(100.0)).is_empty());
        assert_eq!(
            normalize_segments(&[raw(90.0, 120.0)], None),
            [SponsorBlockSegment { start: 90.0, end: 120.0 }]
        );
        assert_eq!(
            normalize_segments(&[raw(90.0, 120.0)], Some(f64::NAN)),
            [SponsorBlockSegment { start: 90.0, end: 120.0 }]
        );
    }

    #[test]
    fn snapshot_hash_tracks_only_canonical_processing_inputs() {
        let segments = [SponsorBlockSegment { start: 10.0, end: 20.0 }];
        let equivalent = [SponsorBlockSegment { start: 10.0, end: 20.0 }];
        assert_eq!(snapshot_hash(&segments), snapshot_hash(&equivalent));
        assert_ne!(
            snapshot_hash(&segments),
            snapshot_hash(&[SponsorBlockSegment { start: 10.0, end: 21.0 }])
        );
        assert_ne!(
            snapshot_hash_for(&segments, &["sponsor"], 1),
            snapshot_hash_for(&segments, &["sponsor", "intro"], 1)
        );
        assert_ne!(
            snapshot_hash_for(&segments, &["sponsor"], 1),
            snapshot_hash_for(&segments, &["sponsor"], 2)
        );
        assert_eq!(snapshot_hash(&segments).len(), 64);
    }

    #[test]
    fn retained_intervals_cover_all_sponsor_positions() {
        assert_eq!(
            retained_intervals(&[SponsorBlockSegment { start: 0.0, end: 10.0 }], 100.0),
            [SponsorBlockSegment { start: 10.0, end: 100.0 }]
        );
        assert_eq!(
            retained_intervals(&[SponsorBlockSegment { start: 40.0, end: 60.0 }], 100.0),
            [
                SponsorBlockSegment { start: 0.0, end: 40.0 },
                SponsorBlockSegment { start: 60.0, end: 100.0 },
            ]
        );
        assert_eq!(
            retained_intervals(&[SponsorBlockSegment { start: 90.0, end: 100.0 }], 100.0),
            [SponsorBlockSegment { start: 0.0, end: 90.0 }]
        );
        assert_eq!(
            retained_intervals(
                &[
                    SponsorBlockSegment { start: 10.0, end: 20.0 },
                    SponsorBlockSegment { start: 30.0, end: 40.0 },
                ],
                50.0,
            ),
            [
                SponsorBlockSegment { start: 0.0, end: 10.0 },
                SponsorBlockSegment { start: 20.0, end: 30.0 },
                SponsorBlockSegment { start: 40.0, end: 50.0 },
            ]
        );
        assert!(retained_intervals(
            &[SponsorBlockSegment { start: 0.0, end: 100.0 }],
            100.0,
        )
        .is_empty());
    }

    #[test]
    fn ffconcat_manifest_references_each_retained_interval() {
        let retained = [
            SponsorBlockSegment { start: 0.0, end: 10.0 },
            SponsorBlockSegment { start: 20.0, end: 30.0 },
        ];
        let manifest = ffconcat_manifest(Path::new("audio/original.mp3"), &retained);
        assert_eq!(
            manifest,
            "ffconcat version 1.0\nfile 'audio/original.mp3'\ninpoint 0.000000\noutpoint 10.000000\nfile 'audio/original.mp3'\ninpoint 20.000000\noutpoint 30.000000\n"
        );
    }

    #[actix_web::test]
    async fn generates_and_probes_a_stream_copy_derivative() {
        let dir = media_fixture_dir();
        let original = dir.join("video-id.mp3");
        let output = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "sine=frequency=440:duration=3", "-q:a", "4", "-y"])
            .arg(&original)
            .output()
            .expect("start fixture FFmpeg");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let original_bytes = fs::read(&original).expect("read original fixture");

        let processed = generate_processed_mp3(
            &original,
            &[SponsorBlockSegment { start: 1.0, end: 2.0 }],
            3.0,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .await
        .expect("generate derivative");

        assert_eq!(processed.filename, "video-id.sponsorblock.0123456789abcdef.mp3");
        assert!((1.8..=2.2).contains(&processed.duration), "duration {}", processed.duration);
        assert_eq!(fs::read(&original).unwrap(), original_bytes);
        assert!(dir.join(processed.filename).is_file());
        assert_eq!(
            fs::read_dir(&dir)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp"))
                .count(),
            0
        );
        fs::remove_dir_all(dir).expect("remove media fixture directory");
    }

    #[actix_web::test]
    async fn generates_derivative_from_a_nested_relative_path() {
        let relative_dir = PathBuf::from("target").join(format!(
            "sponsorblock-relative-{}",
            rand::random::<u64>()
        ));
        fs::create_dir_all(&relative_dir).expect("create relative fixture directory");
        let original = relative_dir.join("video-id.mp3");
        let output = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "sine=frequency=440:duration=3", "-q:a", "4", "-y"])
            .arg(&original)
            .output()
            .expect("start fixture FFmpeg");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));

        let processed = generate_processed_mp3(
            &original,
            &[SponsorBlockSegment { start: 1.0, end: 2.0 }],
            3.0,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .await
        .expect("generate derivative from relative path");

        assert!(relative_dir.join(processed.filename).is_file());
        fs::remove_dir_all(relative_dir).expect("remove relative fixture directory");
    }

    #[actix_web::test]
    async fn rejects_full_coverage_without_publishing_silence() {
        let dir = media_fixture_dir();
        let original = dir.join("video-id.mp3");
        fs::write(&original, b"original").expect("write original fixture");
        let result = generate_processed_mp3(
            &original,
            &[SponsorBlockSegment { start: 0.0, end: 3.0 }],
            3.0,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .await;
        assert!(result.is_err());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).expect("remove media fixture directory");
    }

    #[actix_web::test]
    async fn reconciliation_preserves_active_state_on_failure_and_clears_it_on_empty() {
        let (pool, episode, dir) = reconciliation_fixture().await;
        let raw_segments = [raw(1.0, 2.0)];
        let segments = normalize_segments(&raw_segments, Some(3.0));
        let hash = snapshot_hash(&segments);
        let active_filename = format!("video-id.sponsorblock.{}.mp3", &hash[..16]);
        fs::write(dir.join(&active_filename), b"active derivative").unwrap();
        SponsorBlockCache::upsert_success(
            &pool,
            episode.id,
            &segments,
            &hash,
            Some(&active_filename),
            Some(2.0),
        )
        .await
        .unwrap();

        let same_body = r#"[{"segment":[1.0,2.0],"category":"sponsor","actionType":"skip"}]"#;
        let same_client = SponsorBlockClient::new(
            &server("200 OK", same_body, Duration::ZERO),
            Duration::from_secs(1),
        );
        let unchanged = reconcile_episode(&pool, &same_client, &episode, &dir).await.unwrap();
        assert_eq!(unchanged.snapshot_hash, hash);
        assert!(dir.join(&active_filename).is_file());

        let changed_body = r#"[{"segment":[0.5,2.0],"category":"sponsor","actionType":"skip"}]"#;
        let changed_client = SponsorBlockClient::new(
            &server("200 OK", changed_body, Duration::ZERO),
            Duration::from_secs(1),
        );
        assert!(reconcile_episode(&pool, &changed_client, &episode, &dir).await.is_err());
        let preserved = SponsorBlockCache::read(&pool, episode.id).await.unwrap().unwrap();
        assert_eq!(preserved.snapshot_hash, hash);
        assert!(preserved.last_error.is_some());
        assert!(dir.join(&active_filename).is_file());

        let empty_client = SponsorBlockClient::new(
            &server("404 Not Found", "{}", Duration::ZERO),
            Duration::from_secs(1),
        );
        let empty = reconcile_episode(&pool, &empty_client, &episode, &dir).await.unwrap();
        assert!(empty.segments.is_empty());
        assert!(empty.processed_filename.is_none());
        assert!(!dir.join(active_filename).exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[actix_web::test]
    async fn reconciliation_publishes_first_snapshot_and_replaces_changed_hash() {
        let (pool, episode, dir) = reconciliation_fixture().await;
        let failed_client = SponsorBlockClient::new(
            &server("500 Internal Server Error", "{}", Duration::ZERO),
            Duration::from_secs(1),
        );
        assert!(reconcile_episode(&pool, &failed_client, &episode, &dir).await.is_err());
        assert_eq!(SponsorBlockCache::read(&pool, episode.id).await.unwrap(), None);

        let original = dir.join("video-id.mp3");
        let fixture_output = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "sine=frequency=440:duration=3", "-q:a", "4", "-y"])
            .arg(&original)
            .output()
            .expect("start fixture FFmpeg");
        assert!(fixture_output.status.success());

        let first_body = r#"[{"segment":[1.0,2.0],"category":"sponsor","actionType":"skip"}]"#;
        let first_client = SponsorBlockClient::new(
            &server("200 OK", first_body, Duration::ZERO),
            Duration::from_secs(1),
        );
        let first = reconcile_episode(&pool, &first_client, &episode, &dir).await.unwrap();
        let first_filename = first.processed_filename.clone().expect("first derivative");
        assert!(dir.join(&first_filename).is_file());

        let changed_body = r#"[{"segment":[0.5,1.5],"category":"sponsor","actionType":"skip"}]"#;
        let changed_client = SponsorBlockClient::new(
            &server("200 OK", changed_body, Duration::ZERO),
            Duration::from_secs(1),
        );
        let changed = reconcile_episode(&pool, &changed_client, &episode, &dir).await.unwrap();
        let changed_filename = changed.processed_filename.expect("changed derivative");
        assert_ne!(changed.snapshot_hash, first.snapshot_hash);
        assert_ne!(changed_filename, first_filename);
        assert!(dir.join(changed_filename).is_file());
        assert!(!dir.join(first_filename).exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn parses_episode_duration_strings() {
        assert_eq!(parse_duration_seconds("01:02:03"), Some(3723.0));
        assert_eq!(parse_duration_seconds("02:03"), Some(123.0));
        assert_eq!(parse_duration_seconds("3.5"), Some(3.5));
        assert_eq!(parse_duration_seconds(""), None);
        assert_eq!(parse_duration_seconds("00:00:00"), None);
    }

    #[actix_web::test]
    async fn fetches_matching_response_from_configured_server() {
        let body = r#"[{"segment":[10.0,20.0],"category":"sponsor","actionType":"skip"}]"#;
        let client = SponsorBlockClient::new(
            &server("200 OK", body, Duration::ZERO),
            Duration::from_secs(1),
        );
        let segments = client.fetch("video-id").await.expect("fetch segments");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].segment, [10.0, 20.0]);
    }

    #[actix_web::test]
    async fn treats_not_found_as_an_empty_snapshot() {
        let client = SponsorBlockClient::new(
            &server("404 Not Found", "{}", Duration::ZERO),
            Duration::from_secs(1),
        );
        assert!(client.fetch("video-id").await.expect("empty snapshot").is_empty());
    }

    #[actix_web::test]
    async fn rejects_timeout_malformed_rate_limit_and_server_error() {
        let cases = [
            ("200 OK", "not-json", Duration::ZERO, Duration::from_secs(1)),
            ("429 Too Many Requests", "{}", Duration::ZERO, Duration::from_secs(1)),
            ("500 Internal Server Error", "{}", Duration::ZERO, Duration::from_secs(1)),
            ("200 OK", "[]", Duration::from_millis(100), Duration::from_millis(20)),
        ];
        for (status, body, delay, timeout) in cases {
            let client = SponsorBlockClient::new(&server(status, body, delay), timeout);
            assert!(client.fetch("video-id").await.is_err(), "status {status}");
        }
    }
}