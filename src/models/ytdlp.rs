use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
};
use tokio::process::Command;
use serde::{
    Serialize,
    Deserialize,
    de::Deserializer,
};
use tracing::{
    info,
    debug,
};
// `timestamp_opt` / `Utc` are used only by the `#[cfg(test)]` network smoke
// tests; gated on test builds so the release build stays warning-free.
#[cfg(test)]
use chrono::{Utc, TimeZone};
use super::Error;
use crate::utils::throttle::with_youtube_slot;

pub struct Ytdlp{
    path: String,
    cookies: String,
    runner: Arc<dyn CommandRunner>,
}

struct CommandOutput {
    success: bool,
    code: Option<i32>,
    stdout: Vec<u8>,
}

type CommandFuture<'a> =
    Pin<Box<dyn Future<Output = Result<CommandOutput, std::io::Error>> + Send + 'a>>;

trait CommandRunner: Send + Sync {
    fn run<'a>(&'a self, path: &'a str, args: &'a [&'a str]) -> CommandFuture<'a>;
}

struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn run<'a>(&'a self, path: &'a str, args: &'a [&'a str]) -> CommandFuture<'a> {
        Box::pin(run_streamed(path, args))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YtVideo{
    pub id: String,
    pub title: String,
    #[serde(default, deserialize_with = "string_or_default")]
    pub description: String,
    #[serde(default, deserialize_with = "string_or_default")]
    pub thumbnail: String,
    #[serde(default, deserialize_with = "string_or_default")]
    pub original_url: String,
    #[serde(default, deserialize_with = "string_or_default")]
    pub webpage_url: String,
    #[serde(default, deserialize_with = "string_or_default")]
    pub upload_date: String,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default, deserialize_with = "string_or_default")]
    pub duration_string: String,
    #[serde(default, deserialize_with = "string_or_default")]
    pub release_date: String,
    #[serde(default, deserialize_with = "string_or_default")]
    pub live_status: String,
}

// yt-dlp flat entries and info dicts frequently emit explicit `null` for
// optional string fields (e.g. `"description": null`, `"release_date": null`).
// `#[serde(default)]` alone only covers a missing key, so these deserialize
// `null` as an empty string instead of failing the whole listing parse.
fn string_or_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

impl Ytdlp {
    pub fn new(path: &str, cookies: &str) -> Self{
        info!("new");
        Self{
            path: path.to_string(),
            cookies: cookies.to_string(),
            runner: Arc::new(ProcessCommandRunner),
        }
    }

    #[cfg(test)]
    fn with_runner(path: &str, cookies: &str, runner: Arc<dyn CommandRunner>) -> Self {
        Self {
            path: path.to_string(),
            cookies: cookies.to_string(),
            runner,
        }
    }


    // Runs a yt-dlp command streaming its stderr to the DEBUG log (so long
    // silent runs are observable: progress lines like `[youtube] Extracting
    // URL ...` appear as they are produced) while collecting stdout for
    // parsing. Both pipes are drained concurrently so a large stdout cannot
    // deadlock the stderr stream (or vice versa).
    // Shared cookie flags: listing and download must never diverge on
    // credentials again, or restricted content silently vanishes from scans
    // (youtube-scan-reliability).
    fn cookies_args(&self) -> Vec<&str> {
        if self.cookies.is_empty() {
            vec![]
        } else {
            vec!["--cookies", self.cookies.as_str()]
        }
    }
    pub async fn list_videos(&self, url: &str, want: usize) -> Result<Vec<YtVideo>, Error>{
        info!("list_videos");
        // Flat, bounded listing: the channel `/videos` tab is newest-first, so
        // the first `want` flat entries are "the most recent `want` videos".
        // `--playlist-items` caps the pages yt-dlp walks; the worker further
        // selects the `max`-sized window (scalable-channel-listing).
        let spec = format!("1:{want}");
        let mut args = vec!["--flat-playlist", "--dump-json",
            "--playlist-items", &spec, "--js-runtimes", "node",
            "--js-runtimes", "deno"];
        args.extend(self.cookies_args());
        args.push(url);
        // yt-dlp listing is a YouTube connection: it runs through the shared
        // throttle so it serializes with downloads, the update check, and
        // metadata/image fetches (youtube-throttling). stderr is streamed to
        // the DEBUG log so a long listing stays observable.
        let stdout = with_youtube_slot(move || async move {
            self.runner.run(&self.path, &args).await.map(|output| output.stdout)
        })
        .await?;
        let ytvideos = parse_dump_output(&stdout)?;
        info!(
            "Listed {} flat candidates for {} (requested {})",
            ytvideos.len(),
            &url,
            want
        );
        Ok(ytvideos)
    }

    pub async fn download(&self, id: &str, output: &str) -> Result<(bool, YtVideo), Error>{
        let url = format!("https://www.youtube.com/watch?v={}", id);
        let mut args = vec!["-f", "ba", "-x", "--audio-format", "mp3",
            "--audio-quality", "160K",
            "-o", output, "--print-json", "--js-runtimes", "node",
            "--js-runtimes", "deno", "--retries", "10",
            "--retry-sleep", "5"];
        args.extend(self.cookies_args());
        args.push(&url);
        // One run carries both the download and the full episode metadata
        // (`--print-json`): no separate extraction pass, still under the
        // single YouTube throttle (scalable-channel-listing). stderr is
        // streamed to the DEBUG log (progress during downloads).
        let download_output = with_youtube_slot(move || async move {
            self.runner.run(&self.path, &args).await
        })
        .await
        .map_err(|e| Error::default(&e.to_string()))?;
        let mut videos = parse_dump_output(&download_output.stdout)?;
        let info = match videos.pop() {
            Some(video) => video,
            None => {
                return Err(Error::default(&format!(
                    "yt-dlp produced no metadata for {url} (exit {:?})",
                    download_output.code
                )))
            }
        };
        Ok((download_output.success, info))
    }

    pub async fn metadata(&self, id: &str) -> Result<YtVideo, Error> {
        let url = format!("https://www.youtube.com/watch?v={}", id);
        let mut args = vec!["--skip-download", "--print-json", "--js-runtimes", "node",
            "--js-runtimes", "deno", "--retries", "10", "--retry-sleep", "5"];
        args.extend(self.cookies_args());
        args.push(&url);
        let output = with_youtube_slot(move || async move {
            self.runner.run(&self.path, &args).await
        })
        .await
        .map_err(|e| Error::default(&e.to_string()))?;
        let mut videos = parse_dump_output(&output.stdout)?;
        videos.pop().ok_or_else(|| Error::default(&format!(
            "yt-dlp produced no metadata for {url} (exit {:?})",
            output.code
        )))
    }

    pub async fn auto_update() -> Result<(), Error>{
        let python3 = "python3";
        let args = vec!["-m", "pip", "install", "--user", "--upgrade",
            "--break-system-packages", "yt-dlp[default]"];
        // Async process wait (non-blocking-update-paths): the pip run can take
        // tens of seconds; a synchronous `std::process::Command::wait()` here
        // would pin a tokio worker thread for the whole time.
        //
        // Gated by the shared YouTube throttle too (the update check hits
        // GitHub, but every yt-dlp execution passes through the same slot so a
        // future update channel cannot bypass the throttle).
        let status = with_youtube_slot(|| async move {
            ProcessCommandRunner.run(python3, &args).await.map(|output| output.success)
        })
        // Only Send types cross the slot (io::Error / ExitStatus); the crate
        // `Error` (non-Send) is rebuilt after the cooldown releases.
        .await
        .map_err(|e| Error::default(&e.to_string()))?;
        if status {
            Ok(())
        } else {
            Err(Error::default("Can't update yt-dlp"))
        }
    }
}

async fn run_streamed(
    path: &str,
    args: &[&str],
) -> Result<CommandOutput, std::io::Error> {
    use std::process::Stdio;
    use tokio::io::{
        AsyncBufReadExt,
        AsyncReadExt,
        BufReader,
    };

    let mut child = Command::new(path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");

    let stdout_task = tokio::spawn(async move {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes).await;
        bytes
    });
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            debug!("yt-dlp: {line}");
        }
    });

    let status = child.wait().await?;
    let stdout = stdout_task.await.map_err(std::io::Error::other)?;
    let _ = stderr_task.await;
    Ok(CommandOutput {
        success: status.success(),
        code: status.code(),
        stdout,
    })
}

// Parses `yt-dlp --dump-json` stdout into a vector of videos. yt-dlp prints one
// JSON object per line; the lines are joined inside a top-level array. Empty or
// whitespace-only output yields an empty vector. Kept as a free function so the
// parsing can be unit-tested offline (the network tests for it were flaky).
fn parse_dump_output(stdout: &[u8]) -> Result<Vec<YtVideo>, Error> {
    let lines = std::str::from_utf8(stdout)?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<&str>>()
        .join(",");
    let content = format!("[{}]", lines);
    serde_json::from_str(&content)
        .map_err(|e| Error::default(&format!("Cant parse yt-dlp output: {e}")))
}

#[cfg(test)]
mod flat_listing_tests {
    use super::*;
    use std::time::Duration;

    struct ListingRunner {
        stdout: Vec<u8>,
    }

    impl CommandRunner for ListingRunner {
        fn run<'a>(&'a self, _path: &'a str, _args: &'a [&'a str]) -> CommandFuture<'a> {
            Box::pin(async move {
                Ok(CommandOutput {
                    success: true,
                    code: Some(0),
                    stdout: self.stdout.clone(),
                })
            })
        }
    }

    fn listing_output(total: usize, in_window: usize) -> Vec<u8> {
        let mut content = String::new();
        for i in 0..total {
            // Newest-first, like the `/videos` tab: the first `in_window`
            // entries are recent (2026-08-19); the rest are older (2022-04-03)
            // and below the test floor (2023).
            let (timestamp, date) = if i < in_window {
                (1_755_619_200i64, "20260819") // 2026-08-19
            } else {
                (1_648_944_000i64, "20220403") // 2022-04-03
            };
            // Flat entries omit description/duration/thumbnail on purpose.
            content.push_str(&format!(
                "{{\"id\":\"vid_{i}\",\"title\":\"Title {i}\",\"timestamp\":{timestamp},\
                 \"upload_date\":\"{date}\",\"webpage_url\":\"https://youtu.be/vid_{i}\"}}\n"
            ));
        }
        content.into_bytes()
    }

    #[tokio::test]
    async fn flat_listing_parses_every_entry_tolerantly() {
        crate::utils::throttle::init_throttle(Duration::from_millis(30));
        let runner = Arc::new(ListingRunner {
            stdout: listing_output(300, 50),
        });
        let ytdlp = Ytdlp::with_runner("mock-yt-dlp", "", runner);
        let videos = ytdlp
            .list_videos("https://youtu.be/channel", 300)
            .await
            .expect("flat listing succeeds");
        // Every flat entry parses; omitted fields default to empty.
        assert_eq!(videos.len(), 300, "flat listing must parse every entry");
        assert!(videos.iter().all(|v| v.description.is_empty()));
        assert!(videos.iter().all(|v| v.duration_string.is_empty()));
        assert!(videos.iter().all(|v| v.live_status.is_empty()));
        assert!(videos.iter().all(|v| v.release_date.is_empty()));

        // Count-window selection on top of the listing: newest-first order,
        // floor 2023 → the 50 in-window entries are selected and the scan
        // stops when the first pre-floor entry appears.
        let floor = chrono::TimeZone::timestamp_opt(&Utc, 1_672_531_200, 0).unwrap();
        let now = chrono::TimeZone::timestamp_opt(&Utc, 1_755_619_200, 0).unwrap();
        let selection = crate::utils::worker::select_window(videos, 50, floor, now);
        assert_eq!(
            selection.window.len(),
            50,
            "the 50 most recent candidates form the window"
        );
        for (offset, candidate) in selection.window.iter().enumerate() {
            assert_eq!(candidate.id, format!("vid_{offset}"));
        }

    }

    #[test]
    fn print_json_info_dict_parses_full_metadata() {
        // Simulates the stdout of `yt-dlp --print-json` during a download.
        let line = br#"{"id":"abc123","title":"Full Episode","description":"desc","thumbnail":"https://x/t.jpg","original_url":"https://youtu.be/abc123","webpage_url":"https://youtu.be/abc123","upload_date":"20260819","timestamp":1755619200,"duration_string":"00:05:30"}"#;
        let videos = parse_dump_output(line).expect("info dict parses");
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].title, "Full Episode");
        assert_eq!(videos[0].description, "desc");
        assert_eq!(videos[0].duration_string, "00:05:30");
        assert_eq!(videos[0].timestamp, Some(1_755_619_200));
    }
}

#[cfg(test)]
mod parse_dump_output_tests {
    use super::parse_dump_output;

    const SAMPLE: &[u8] = br#"{"id":"aaa111","title":"Video A","duration_string":"00:01:00","upload_date":"20260101","webpage_url":"https://youtu.be/aaa111","thumbnail":"https://x/a.jpg","original_url":"https://youtu.be/aaa111","description":"desc A"}
{"id":"bbb222","title":"Video B","duration_string":"00:02:30","upload_date":"20260102","webpage_url":"https://youtu.be/bbb222","thumbnail":"https://x/b.jpg","original_url":"https://youtu.be/bbb222","description":"desc B"}"#;

    #[test]
    fn parses_multiple_dump_json_lines() {
        let videos = parse_dump_output(SAMPLE).expect("valid output parses");
        assert_eq!(videos.len(), 2);
        assert_eq!(videos[0].id, "aaa111");
        assert_eq!(videos[0].title, "Video A");
        assert_eq!(videos[1].duration_string, "00:02:30");
        assert_eq!(videos[1].description, "desc B");
    }

    #[test]
    fn optional_timestamp_defaults_to_none() {
        let videos = parse_dump_output(SAMPLE).expect("valid output parses");
        assert_eq!(videos[0].timestamp, None);
    }

    #[test]
    fn empty_output_yields_empty_vector() {
        let videos = parse_dump_output(b"").expect("empty output is valid");
        assert!(videos.is_empty());
    }

    #[test]
    fn whitespace_output_yields_empty_vector() {
        let videos = parse_dump_output(b"\n  \n\t\n").expect("whitespace is valid");
        assert!(videos.is_empty());
    }

    #[test]
    fn non_json_output_errors() {
        assert!(parse_dump_output(b"not json at all\n").is_err());
    }
}

#[cfg(test)]
mod download_args_tests {
    use super::*;
    use std::{sync::Mutex, time::Duration};

    struct ArgumentsRunner {
        args: Mutex<Vec<String>>,
    }

    impl CommandRunner for ArgumentsRunner {
        fn run<'a>(&'a self, _path: &'a str, args: &'a [&'a str]) -> CommandFuture<'a> {
            *self.args.lock().unwrap() = args.iter().map(|arg| (*arg).to_string()).collect();
            Box::pin(async {
                Ok(CommandOutput {
                    success: true,
                    code: Some(0),
                    stdout: br#"{"id":"video-id","title":"Video","description":"","thumbnail":"","original_url":"https://youtu.be/video-id","webpage_url":"https://youtu.be/video-id","upload_date":"20260828","duration_string":"00:01:00"}"#.to_vec(),
                })
            })
        }
    }

    #[tokio::test]
    async fn download_requests_constant_bitrate_mp3() {
        crate::utils::throttle::init_throttle(Duration::ZERO);
        let runner = Arc::new(ArgumentsRunner { args: Mutex::new(Vec::new()) });
        let ytdlp = Ytdlp::with_runner("mock-yt-dlp", "", runner.clone());

        ytdlp.download("video-id", "audio.mp3").await.unwrap();

        let args = runner.args.lock().unwrap();
        assert!(args.windows(2).any(|pair| pair == ["--audio-quality", "160K"]));
    }

    #[tokio::test]
    async fn metadata_probe_skips_media_download() {
        crate::utils::throttle::init_throttle(Duration::ZERO);
        let runner = Arc::new(ArgumentsRunner { args: Mutex::new(Vec::new()) });
        let ytdlp = Ytdlp::with_runner("mock-yt-dlp", "", runner.clone());

        let metadata = ytdlp.metadata("video-id").await.unwrap();

        assert_eq!(metadata.upload_date, "20260828");
        let args = runner.args.lock().unwrap();
        assert!(args.iter().any(|arg| arg == "--skip-download"));
        assert!(!args.iter().any(|arg| arg == "-x" || arg == "-o"));
    }
}

#[cfg(test)]
mod throttle_youtubedl_integration {
    use super::*;
    use crate::utils::throttle::init_throttle;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::{Duration, Instant},
    };

    struct RecordingRunner {
        active: AtomicUsize,
        max_active: AtomicUsize,
        events: std::sync::Mutex<Vec<(char, Instant)>>,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                active: AtomicUsize::new(0),
                max_active: AtomicUsize::new(0),
                events: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for RecordingRunner {
        fn run<'a>(&'a self, _path: &'a str, _args: &'a [&'a str]) -> CommandFuture<'a> {
            Box::pin(async move {
                self.events.lock().unwrap().push(('s', Instant::now()));
                let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
                self.max_active.fetch_max(active, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(50)).await;
                self.active.fetch_sub(1, Ordering::SeqCst);
                self.events.lock().unwrap().push(('e', Instant::now()));
                Ok(CommandOutput {
                    success: true,
                    code: Some(0),
                    stdout: br#"{"id":"x","title":"Mock Video","description":"d","thumbnail":"","original_url":"http://x","webpage_url":"http://x","upload_date":"20260101","duration_string":"00:01:00"}"#.to_vec(),
                })
            })
        }
    }

    #[tokio::test]
    async fn concurrent_downloads_are_sequential_and_respect_cooldown() {
        init_throttle(Duration::from_millis(30));
        let runner = Arc::new(RecordingRunner::new());
        let ytdlp = Arc::new(Ytdlp::with_runner(
            "mock-yt-dlp",
            "",
            runner.clone(),
        ));
        let mut handles = Vec::new();
        for i in 0..4 {
            let ytdlp = std::sync::Arc::clone(&ytdlp);
            handles.push(tokio::spawn(async move {
                let (success, _info) = ytdlp
                    .download(&format!("id{i}"), "unused-output.mp3")
                    .await
                    .expect("mock yt-dlp succeeds");
                success
            }));
        }
        for handle in handles {
            assert!(
                handle.await.expect("task did not panic"),
                "every fake download must exit successfully"
            );
        }

        assert_eq!(runner.max_active.load(Ordering::SeqCst), 1);
        let events = runner.events.lock().unwrap();
        assert_eq!(events.len(), 8, "4 runs × (start, end) events");

        let mut previous_end: Option<Instant> = None;
        for chunk in events.chunks(2) {
            assert_eq!(chunk[0].0, 's', "runs must not overlap (double start)");
            assert_eq!(chunk[1].0, 'e', "run must end before the next starts");
            let next_start = chunk[0].1;
            if let Some(previous_end_at) = previous_end {
                assert!(
                    next_start.duration_since(previous_end_at) >= Duration::from_millis(20),
                    "cooldown was not enforced between command executions"
                );
            }
            previous_end = Some(chunk[1].1);
        }
    }
}

#[test]
    fn null_string_fields_default_to_empty() {
        // yt-dlp flat entries emit explicit `null` for optional strings.
        let line = br#"{"id":"n1","title":"T","description":null,"thumbnail":null,"original_url":null,"webpage_url":"https://x","upload_date":null,"timestamp":null,"duration_string":null,"release_date":null,"live_status":null}"#;
        let videos = parse_dump_output(line).expect("null fields must not fail the parse");
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].description, "");
        assert_eq!(videos[0].duration_string, "");
        assert_eq!(videos[0].release_date, "");
        assert_eq!(videos[0].live_status, "");
        assert_eq!(videos[0].webpage_url, "https://x");
    }

    #[test]
    fn real_atareao_flat_listing_parses() {
        // Fixture captured from `yt-dlp --flat-playlist --dump-json
        // --playlist-items 1:12 https://www.youtube.com/@atareao/videos`
        // (2026.08.19) — the exact shape that broke parsing: explicit
        // `"live_status": null` / `"timestamp": null` plus absent
        // description/date fields.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/atareao_flat_first.jsonl");
        let bytes = std::fs::read(path).expect("fixture file");
        let videos = parse_dump_output(&bytes).expect("real flat listing must parse");
        assert_eq!(videos.len(), 12);
        assert!(videos.iter().all(|v| v.live_status.is_empty()));
        assert!(videos.iter().all(|v| v.timestamp.is_none()));
        assert!(videos.iter().all(|v| v.id.len() == 11), "yt video ids are 11 chars");
        assert!(!videos[0].title.is_empty());
        assert_eq!(videos[0].id, "FuxbDWsB6so");
    }

    #[tokio::test]
    async fn test_e(){
    let ytdlp = Ytdlp::new("yt-dlp", "cookies.txt");
    // Old date: the "error" channel yields no parseable videos.
    let _old = Utc.timestamp_opt(0, 0).unwrap();
    let salida = ytdlp.list_videos("error", 5).await;
    match salida{
        Ok(videos) => {
            assert!(videos.is_empty());
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
#[tokio::test]
async fn test_0(){
    let ytdlp = Ytdlp::new("yt-dlp", "cookies.txt");
    // Recent date: expect no/very few videos on this channel.
    let salida = ytdlp.list_videos("atareao", 5).await;
    match salida{
        Ok(videos) => {
            assert!(videos.is_empty());
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
#[tokio::test]
async fn test_ytdlp(){
    let ytdlp = Ytdlp::new("yt-dlp", "cookies.txt");
    let salida = ytdlp.download("mWoJw5qD0eI", "/tmp/test.mp3").await;
    println!("{:?}", salida);
}

