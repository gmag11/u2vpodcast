use regex::Regex;
use ureq::Agent;
use std::sync::OnceLock;
use std::time::Duration;

use super::Error;

// Upper bound for the metadata fetch so a hung upstream cannot stall blocking
// threads (or the async workers waiting on them) indefinitely.
const METADATA_TIMEOUT: Duration = Duration::from_secs(30);



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
