use regex::Regex;
use ureq::Agent;
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

fn get_image(html: &str) -> String{
    let pattern = r#"meta\s+property="og:image"\s+content="(?P<content>[^"]*)""#;
    let re = Regex::new(pattern).unwrap();
    re.captures(html)
        .map(|c| {
            let part = c["content"].to_string();
            part.find('?')
            .map(|pos| part[..pos].to_string())
            .unwrap_or(part)
        })
        .unwrap_or("".to_string())
}

fn get_metadata(html: &str, metadata: &str) -> String{
    let pattern = format!(r#"meta\s+property="{}"\s+content="(?P<content>[^"]*)""#,
        metadata);
    let re = Regex::new(&pattern).unwrap();
    re.captures(html)
        .map(|c| c["content"].to_string())
        .unwrap_or("".to_string())
        
}



#[tokio::test]
async fn test_info_channel(){
    let url = "https://www.youtube.com/c/atareao";
    let ytinfo = YTInfo::new(url).await;
    println!("{:?}", ytinfo);
    assert!(ytinfo.is_ok())
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
