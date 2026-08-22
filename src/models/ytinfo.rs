use regex::Regex;
use ureq::Agent;
use std::sync::OnceLock;
use std::time::Duration;
use tracing::{info, warn};

use super::{
    Error,
    images_dir,
};

// Upper bound for the metadata fetch so a hung upstream cannot stall blocking
// threads (or the async workers waiting on them) indefinitely.
const METADATA_TIMEOUT: Duration = Duration::from_secs(30);

// Upper bound for a cached cover image. Covers are small JPEGs; any body
// beyond this cap is treated as a failed download and the previous cached
// file (if any) is kept (channel-image-cache / disk-growth risk).
const IMAGE_MAX_BYTES: u64 = 5 * 1024 * 1024;

// Bounded timeout for the image probe/download, like the metadata fetch, so a
// hung upstream cannot stall blocking threads indefinitely.
const IMAGE_TIMEOUT: Duration = Duration::from_secs(30);

// Stable local URL for a channel's cached cover; derived from the slug so it
// stays the same across API responses until the cache is refreshed. The
// filenames are slug-derived (`[a-z0-9_]+`), so no URL escaping is needed.
pub fn image_local_url(slug: &str) -> String {
    format!("/images/{slug}.jpg")
}



#[derive(Debug, Clone)]
pub struct YTInfo{
    pub title: String,
    pub description: String,
    pub image: String,
}

impl YTInfo{
    pub fn default() -> Self {
        Self{
            title: "".to_string(),
            description: "".to_string(),
            image: "".to_string(),
        }
    }

    pub async fn new(url: &str) -> Result<Self, Error>{

        // The upstream HTTP fetch is fully synchronous; run it on the blocking
        // thread pool so two slow/hung fetches can never stall the few tokio
        // worker threads that serve the whole API. The closure returns a plain
        // String error because this crate's `Error` wraps a non-Send `Session`.
        let url = url.to_string();
        let html = actix_web::rt::task::spawn_blocking(move || -> Result<String, String> {
            let agent: Agent = ureq::Agent::config_builder()
                .timeout_global(Some(METADATA_TIMEOUT))
                .build()
                .into();
            agent.get(&url)
                .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
                .header("Accept-Language", "en-US,en;q=0.9")
                .call()
                .map_err(|e| e.to_string())?
                .body_mut()
                .read_to_string()
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| Error::default(&e.to_string()))?
        .map_err(|e| Error::default(&e))?;

        let title = get_metadata(&html, "og:title");
        let description = get_metadata(&html, "og:description");
        let image = get_image(&html);

        Ok(Self{
            title,
            description,
            image,
        })
    }
}

// Outcome of the blocking probe + download.
#[derive(Debug)]
enum ImageFetchOutcome {
    // An existing cached file already matches the remote `Content-Length` from
    // the HEAD probe; no download is needed.
    Skip,
    // Fresh bytes to store atomically.
    Bytes(Vec<u8>),
}

// Blocking ureq HEAD probe + bounded GET, run inside `spawn_blocking` (see
// `cache_image` below). Uses the exact same fetch mechanism as `YTInfo::new`
// (a ureq agent with a global timeout on the blocking pool), so the
// single-connection YouTube throttle (`limit-youtube-concurrency`) can wrap
// this code path exactly like the metadata fetch once implemented (task 2.5).
fn image_fetch_blocking(dest: &str, remote_url: &str) -> Result<ImageFetchOutcome, String> {
    let agent: Agent = ureq::Agent::config_builder()
        .timeout_global(Some(IMAGE_TIMEOUT))
        .build()
        .into();

    // Size probe first: when a cached file already exists and its on-disk size
    // equals the reported `Content-Length`, skip the download entirely (most
    // sync cycles cost one cheap HEAD). A failed/absent probe or missing
    // `Content-Length` falls through to the bounded GET below.
    let probe_len = match agent.head(remote_url).call() {
        Ok(resp) => resp
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok()),
        Err(_) => None,
    };
    if let Some(reported) = probe_len {
        if let Ok(meta) = std::fs::metadata(dest) {
            if meta.len() == reported {
                return Ok(ImageFetchOutcome::Skip);
            }
        }
    }

    let mut resp = agent
        .get(remote_url)
        .call()
        .map_err(|e| e.to_string())?;
    let bytes = resp
        .body_mut()
        .with_config()
        .limit(IMAGE_MAX_BYTES)
        .read_to_vec()
        .map_err(|e| e.to_string())?;
    if bytes.is_empty() {
        return Err("image body is empty".to_string());
    }
    Ok(ImageFetchOutcome::Bytes(bytes))
}

// Cache a channel's cover image as `{slug}.jpg`: HEAD probe (skip when the
// cached size matches), then a bounded GET with an atomic temp-file + rename
// write. Returns the stable local URL (`/images/{slug}.jpg`) when the cache is
// populated/current, or `None` when there is no remote image or the fetch
// failed — in which case the caller keeps the previous `channel.image`
// untouched (channel-image-cache).
pub async fn cache_image(slug: &str, remote_url: &str) -> Result<Option<String>, Error> {
    cache_image_in_dir(images_dir(), slug, remote_url).await
}

// Directory-injectable variant used by production (`images_dir()`) and by the
// integration tests (an isolated temp directory) so `cache_image`'s full
// probe + download + atomic-write flow is exercised against a real cache dir.
async fn cache_image_in_dir(
    dir: &str,
    slug: &str,
    remote_url: &str,
) -> Result<Option<String>, Error> {
    if remote_url.trim().is_empty() {
        return Ok(None);
    }
    let dest = format!("{dir}/{slug}.jpg");
    let remote_url = remote_url.to_string();
    let tmp = format!(
        "{dest}.{}.{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros())
            .unwrap_or(0)
    );
    let probe_dest = dest.clone();
    let outcome = match actix_web::rt::task::spawn_blocking(move || {
        image_fetch_blocking(&probe_dest, &remote_url)
    })
    .await
    {
        // Probe/download failure (timeout, HTTP error, oversized body, ...):
        // keep the previous cached file and let the caller keep the previous
        // `channel.image` (channel-image-cache).
        Ok(Err(reason)) => {
            warn!("Keeping previous cached image for `{slug}`: {reason}");
            return Ok(None);
        }
        // Blocking-pool join failure: unrecoverable.
        Err(e) => return Err(Error::default(&e.to_string())),
        Ok(Ok(outcome)) => outcome,
    };

    match outcome {
        ImageFetchOutcome::Skip => {
            info!(
                "Cached image for `{slug}` unchanged (probe size matches); skipping download"
            );
            Ok(Some(image_local_url(slug)))
        }
        ImageFetchOutcome::Bytes(bytes) => {
            // Atomic write: temp file + rename so a concurrent reader never
            // sees a half-written image. A unique temp suffix (pid + micros)
            // avoids two concurrent refreshes of the same slug fighting over a
            // single temp path.
            tokio::fs::write(&tmp, &bytes)
                .await
                .map_err(|e| Error::default(&e.to_string()))?;
            tokio::fs::rename(&tmp, &dest)
                .await
                .map_err(|e| Error::default(&e.to_string()))?;
            info!(
                "Cached cover image for `{slug}` ({} bytes)",
                bytes.len()
            );
            Ok(Some(image_local_url(slug)))
        }
    }
}

// The upstream OCR-style og metadata is fetched with attribute order and quote
// style varying in the wild: `property` may come after `content`, attributes
// may use single quotes, and extra attributes can sit between them. The regex
// crate has no lookaround, so we compile one pattern per attribute order and
// scan each until the requested property matches (youtube-scan-reliability).
static META_PROPERTY_FIRST: OnceLock<Regex> = OnceLock::new();
static META_CONTENT_FIRST: OnceLock<Regex> = OnceLock::new();

// <meta ... property="X" ... content="Y" ...>
const PROPERTY_FIRST: &str = r#"(?i)<meta\b[^>]*\bproperty\s*=\s*(?:"(?P<prop>[^"]*)"|'(?P<props>[^']*)')[^>]*\bcontent\s*=\s*(?:"(?P<content>[^"]*)"|'(?P<contents>[^']*)')[^>]*>"#;
// <meta ... content="Y" ... property="X" ...>
const CONTENT_FIRST: &str = r#"(?i)<meta\b[^>]*\bcontent\s*=\s*(?:"(?P<content>[^"]*)"|'(?P<contents>[^']*)')[^>]*\bproperty\s*=\s*(?:"(?P<prop>[^"]*)"|'(?P<props>[^']*)')[^>]*>"#;

fn scan_metadata(re: &Regex, html: &str, key: &str) -> Option<String> {
    for caps in re.captures_iter(html) {
        let prop = caps
            .name("prop")
            .or_else(|| caps.name("props"))
            .map(|m| m.as_str())
            .unwrap_or_default();
        if prop.eq_ignore_ascii_case(key) {
            if let Some(content) = caps
                .name("content")
                .or_else(|| caps.name("contents"))
            {
                return Some(content.as_str().to_string());
            }
        }
    }
    None
}

fn get_image(html: &str) -> String{
    let image = get_metadata(html, "og:image");
    match image.find('?') {
        // og:image URLs carry size/quality query params; strip them.
        Some(pos) => image[..pos].to_string(),
        None => image,
    }
}

fn get_metadata(html: &str, metadata: &str) -> String{
    scan_metadata(META_PROPERTY_FIRST.get_or_init(|| Regex::new(PROPERTY_FIRST).unwrap()), html, metadata)
        .or_else(|| scan_metadata(META_CONTENT_FIRST.get_or_init(|| Regex::new(CONTENT_FIRST).unwrap()), html, metadata))
        .unwrap_or_default()
}

#[cfg(test)]
mod metadata_parsing_tests {
    use super::{get_image, get_metadata};

    #[test]
    fn canonical_double_quoted() {
        let html = r#"<meta property="og:title" content="Canal X">"#;
        assert_eq!(get_metadata(html, "og:title"), "Canal X");
    }

    #[test]
    fn reversed_order_single_quoted() {
        let html = r#"<meta content='Canal Y' property="og:title">"#;
        assert_eq!(get_metadata(html, "og:title"), "Canal Y");
    }

    #[test]
    fn extra_attribute_in_between() {
        let html = r#"<meta property="og:title" data-x="1" content="Zed">"#;
        assert_eq!(get_metadata(html, "og:title"), "Zed");
    }

    #[test]
    fn property_first_single_quotes() {
        let html = r#"<meta property='og:title' content="Alpha">"#;
        assert_eq!(get_metadata(html, "og:title"), "Alpha");
    }

    #[test]
    fn missing_returns_empty() {
        assert_eq!(get_metadata("<html></html>", "og:title"), "");
    }

    #[test]
    fn unclosed_meta_returns_empty() {
        assert_eq!(get_metadata(r#"<meta property="og:title" content="broken"#, "og:title"), "");
    }

    #[test]
    fn case_insensitive_property() {
        let html = r#"<meta property="OG:TITLE" content="Upper">"#;
        assert_eq!(get_metadata(html, "og:title"), "Upper");
    }

    #[test]
    fn wrong_key_is_ignored() {
        let html = r#"<meta property="og:title" content="T"><meta property="og:description" content="D">"#;
        assert_eq!(get_metadata(html, "og:description"), "D");
    }

    #[test]
    fn image_suffix_stripped() {
        let html = r#"<meta property="og:image" content="https://x/img.jpg?w=120&h=120">"#;
        assert_eq!(get_image(html), "https://x/img.jpg");
    }

    #[test]
    fn image_without_params_kept() {
        let html = r#"<meta content="https://x/img.png" property="og:image">"#;
        assert_eq!(get_image(html), "https://x/img.png");
    }

}

#[cfg(test)]
mod image_cache_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "u2v-image-test-{tag}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    // Minimal HTTP server (HEAD + GET) so probe/download logic is exercised
    // against a real TCP conversation instead of YouTube's live CDN. It counts
    // HEAD and GET requests so the integration tests can assert "unchanged
    // image performs no GET" and "changed size triggers one GET".
    #[derive(Default)]
    struct ServerCounters {
        heads: usize,
        gets: usize,
    }

    struct TestServer {
        addr: std::net::SocketAddr,
        counters: Arc<Mutex<ServerCounters>>,
    }

    impl TestServer {
        fn spawn(body: Vec<u8>, fail_head: bool) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
            let addr = listener.local_addr().expect("local addr");
            let counters = Arc::new(Mutex::new(ServerCounters::default()));
            let thread_counters = Arc::clone(&counters);
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else { continue };
                    let mut buf = [0u8; 4096];
                    if stream.read(&mut buf).unwrap_or(0) == 0 {
                        continue;
                    }
                    let request = String::from_utf8_lossy(&buf);
                    let method = request.split_whitespace().next().unwrap_or("").to_string();
                    if method == "HEAD" {
                        thread_counters.lock().unwrap().heads += 1;
                        if fail_head {
                            let _ = stream.write_all(
                                b"HTTP/1.1 503 Service Unavailable\r\nConnection: close\r\n\r\n",
                            );
                            continue;
                        }
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(header.as_bytes());
                    } else {
                        thread_counters.lock().unwrap().gets += 1;
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(header.as_bytes());
                        let _ = stream.write_all(&body);
                    }
                }
            });
            TestServer { addr, counters }
        }

        fn url(&self) -> String {
            format!("http://{}/ch.jpg", self.addr)
        }

        fn heads(&self) -> usize {
            self.counters.lock().unwrap().heads
        }

        fn gets(&self) -> usize {
            self.counters.lock().unwrap().gets
        }
    }

    #[test]
    fn local_url_is_stable_per_slug() {
        assert_eq!(image_local_url("mi_canal"), "/images/mi_canal.jpg");
        assert_eq!(image_local_url("ch-2"), "/images/ch-2.jpg");
    }

    #[tokio::test]
    async fn empty_remote_url_returns_none_without_network() {
        assert!(cache_image("cualquier", "").await.unwrap().is_none());
        assert!(cache_image("cualquier", "   ").await.unwrap().is_none());
    }

    #[test]
    fn skip_when_head_size_matches_cached_file() {
        let body = vec![0xAB; 512];
        let server = TestServer::spawn(body.clone(), false);
        let dir = temp_dir("skip");
        let dest = dir.join("ch.jpg");
        std::fs::write(&dest, &body).expect("seed cached file");
        let url = server.url();
        match image_fetch_blocking(dest.to_str().unwrap(), &url) {
            Ok(ImageFetchOutcome::Skip) => {}
            other => panic!("expected Skip, got {other:?}"),
        }
        // The cached file must be untouched.
        assert_eq!(std::fs::read(&dest).unwrap(), body);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn changed_size_triggers_download() {
        let body = vec![0xAB; 512];
        let server = TestServer::spawn(body.clone(), false);
        let dir = temp_dir("changed");
        let dest = dir.join("ch.jpg");
        std::fs::write(&dest, vec![0u8; 64]).expect("seed smaller file");
        let url = server.url();
        match image_fetch_blocking(dest.to_str().unwrap(), &url) {
            Ok(ImageFetchOutcome::Bytes(bytes)) => assert_eq!(bytes, body),
            other => panic!("expected fresh Bytes, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn download_when_no_cached_file() {
        let body = vec![0xCD; 256];
        let server = TestServer::spawn(body.clone(), false);
        let dir = temp_dir("none");
        let dest = dir.join("ch.jpg"); // does not exist
        let url = server.url();
        match image_fetch_blocking(dest.to_str().unwrap(), &url) {
            Ok(ImageFetchOutcome::Bytes(bytes)) => assert_eq!(bytes, body),
            other => panic!("expected Bytes, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn probe_failure_falls_back_to_bounded_download() {
        let body = vec![0xEF; 128];
        let server = TestServer::spawn(body.clone(), true); // HEAD -> 503
        let dir = temp_dir("probefail");
        let dest = dir.join("ch.jpg");
        let url = server.url();
        match image_fetch_blocking(dest.to_str().unwrap(), &url) {
            Ok(ImageFetchOutcome::Bytes(bytes)) => assert_eq!(bytes, body),
            other => panic!("expected Bytes after probe failure, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversized_body_is_rejected() {
        let body = vec![0x11u8; (IMAGE_MAX_BYTES + 1024) as usize];
        let server = TestServer::spawn(body, false);
        let dir = temp_dir("oversize");
        let dest = dir.join("ch.jpg");
        let url = server.url();
        let result = image_fetch_blocking(dest.to_str().unwrap(), &url);
        assert!(result.is_err(), "oversized body must be rejected");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- Integration tests (channel-image-cache tasks 3.2 / 3.4) ----
    //
    // These exercise the full `cache_image` pipeline (probe + bounded GET +
    // atomic write on a real cache directory) against a local HTTP server,
    // asserting the wire behavior spec requires: an unchanged image performs
    // no download (only HEAD), a changed image is re-downloaded, and a failed
    // refresh keeps the previous file untouched (persistence).

    #[tokio::test]
    async fn integ_unchanged_image_performs_no_download() {
        let body = vec![0x42; 1024];
        let server = TestServer::spawn(body.clone(), false);
        let dir = temp_dir("integ-skip");
        let slug = "mi_canal";
        let url = server.url();

        // First fetch: HEAD probe + one bounded GET, file lands in the cache.
        let local = cache_image_in_dir(dir.to_str().unwrap(), slug, &url)
            .await
            .expect("first fetch must succeed");
        assert_eq!(local.as_deref(), Some("/images/mi_canal.jpg"));
        assert_eq!(server.heads(), 1, "one HEAD probe on first fetch");
        assert_eq!(server.gets(), 1, "one GET download on first fetch");
        let path = dir.join("mi_canal.jpg");
        let file = std::fs::read(&path).expect("cached file on disk");
        assert_eq!(file, body);

        // Second fetch, same remote size: HEAD only, no new GET.
        let local2 = cache_image_in_dir(dir.to_str().unwrap(), slug, &url)
            .await
            .expect("second fetch must succeed");
        assert_eq!(local2, Some("/images/mi_canal.jpg".to_string()));
        assert_eq!(server.heads(), 2, "second refresh probes again");
        assert_eq!(server.gets(), 1, "unchanged image must NOT be downloaded again");
        assert_eq!(std::fs::read(&path).unwrap(), file, "cached file untouched");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn integ_changed_size_triggers_redownload() {
        let body = vec![0x42; 1024];
        let server = TestServer::spawn(body.clone(), false);
        let dir = temp_dir("integ-changed");
        let slug = "otro_canal";
        let url = server.url();

        cache_image_in_dir(dir.to_str().unwrap(), slug, &url)
            .await
            .expect("first fetch");
        assert_eq!(server.gets(), 1);

        // Remote image grows: a second server serves a different size, and the
        // refresh must replace the cached file.
        let bigger = vec![0x43; 4096];
        let server2 = TestServer::spawn(bigger.clone(), false);
        let local = cache_image_in_dir(dir.to_str().unwrap(), slug, &server2.url())
            .await
            .expect("refresh after size change");
        assert_eq!(local.as_deref(), Some("/images/otro_canal.jpg"));
        let path = dir.join("otro_canal.jpg");
        assert_eq!(std::fs::read(&path).unwrap(), bigger, "cached file replaced");
        assert_eq!(server2.gets(), 1, "exactly one GET for the changed image");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn integ_failed_refresh_keeps_previous_file() {
        let body = vec![0x99; 300];
        let server = TestServer::spawn(body.clone(), false);
        let dir = temp_dir("integ-fail");
        let slug = "tercero";
        let url = server.url();

        cache_image_in_dir(dir.to_str().unwrap(), slug, &url)
            .await
            .expect("first fetch");
        let path = dir.join("tercero.jpg");
        let before = std::fs::read(&path).unwrap();

        // Unreachable upstream: probe fails and the fallback GET fails too.
        // The cache must be left intact and `None` returned (previous image
        // URL is kept by callers).
        let dead = "http://127.0.0.1:1/ch.jpg";
        let res = cache_image_in_dir(dir.to_str().unwrap(), slug, dead)
            .await
            .expect("failed refresh must not surface an error");
        assert!(res.is_none(), "failed refresh signals no new local URL");
        assert_eq!(std::fs::read(&path).unwrap(), before, "previous file persisted");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn real_youtube_atareao_image_caches_and_probe_skips_unchanged() {
        // True end-to-end against the atareao channel: fetch its metadata (the
        // same call the app makes), cache the real cover, then re-probe the
        // real CDN. The second probe MUST report Skip (probe size == cached
        // size => no download), which is exactly the 3.2 "skip-if-same"
        // guarantee on the real upstream.
        let ytinfo = YTInfo::new("https://www.youtube.com/c/atareao")
            .await
            .expect("atareao metadata must be fetchable (same as test_info_channel)");
        assert!(
            !ytinfo.image.is_empty(),
            "atareao channel must expose an og:image URL"
        );
        let url = ytinfo.image;
        let dir = temp_dir("real-atareao");
        let dest = dir.join("atareao.jpg");

        let seeded = match image_fetch_blocking(dest.to_str().unwrap(), &url) {
            Ok(ImageFetchOutcome::Bytes(bytes)) => {
                assert!(bytes.len() >= 4, "real cover must not be empty");
                std::fs::write(&dest, &bytes).expect("seed cache from real CDN");
                bytes
            }
            Ok(ImageFetchOutcome::Skip) => {
                // Cannot happen: no cache file existed before the first call.
                panic!("first probe skipped although no cache file existed");
            }
            Err(e) => panic!("real image fetch failed: {e}"),
        };

        let second = image_fetch_blocking(dest.to_str().unwrap(), &url).unwrap();
        match second {
            ImageFetchOutcome::Skip => {}
            ImageFetchOutcome::Bytes(bytes) => {
                panic!(
                    "unchanged real cover was downloaded again ({} != {} bytes)",
                    seeded.len(),
                    bytes.len()
                );
            }
        }
        assert_eq!(
            std::fs::read(&dest).unwrap(),
            seeded,
            "cached file untouched after skip"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}



#[tokio::test]
async fn test_info_channel(){
    let url = "https://www.youtube.com/c/atareao";
    let ytinfo = YTInfo::new(url).await;
    println!("{:?}", ytinfo);
    assert!(ytinfo.is_ok());
    // The robust parser must extract a non-empty title from real YouTube HTML
    // (youtube-scan-reliability); empty titles degrade to generic channel slugs.
    let info = ytinfo.unwrap();
    assert!(!info.title.trim().is_empty(), "title must be parsed from real HTML");
}

#[tokio::test]
async fn test_info_playlist(){
    let url = "https://www.youtube.com/playlist?list=PL3lTiK2rXrUFdTzriDsmNCG28T8u7bhEd";
    let ytinfo = YTInfo::new(url).await;
    println!("{:?}", ytinfo);
    assert!(ytinfo.is_ok())
}

#[tokio::test]
async fn test_info_video(){
    let url = "https://www.youtube.com/watch?v=2A1abiQJAiM";
    let ytinfo = YTInfo::new(url).await;
    println!("{:?}", ytinfo);
    assert!(ytinfo.is_ok())
}
