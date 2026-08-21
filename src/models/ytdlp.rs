use tokio::process::Command;
use serde::{Serialize, Deserialize};
use tracing::{
    info,
};
use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use super::Error;

pub struct Ytdlp{
    path: String,
    cookies: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YtVideo{
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumbnail: String,
    pub original_url: String,
    pub webpage_url: String,
    pub upload_date: String,
    #[serde(default)]
    pub timestamp: Option<i64>,
    pub duration_string: String,
}

impl Ytdlp {
    pub fn new(path: &str, cookies: &str) -> Self{
        info!("new");
        Self{
            path: path.to_string(),
            cookies: cookies.to_string(),
        }
    }
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
    pub async fn get_latest(&self, url: &str, last: DateTime<Utc>) -> Result<Vec<YtVideo>, Error>{
        info!("get_latest");
        // Fetch every video published on or after the date of the last stored
        // episode. Using the absolute date (not "today-Ndays") avoids missing
        // videos published earlier the same day the scheduler/worker runs.
        let elapsed = last.format("%Y%m%d").to_string();
        let mut args = vec!["--dateafter", &elapsed, "--dump-json",
            "--break-on-reject", "--js-runtimes", "node",
            "--js-runtimes", "deno"];
        args.extend(self.cookies_args());
        args.push(url);
        let stdout = Command::new(&self.path)
            .args(&args)
            .output()
            .await?
            .stdout;
        let ytvideos = parse_dump_output(&stdout)?;
        info!("{:?}", &ytvideos);
        Ok(ytvideos)
    }

    pub async fn download(&self, id: &str, output: &str) -> Result<std::process::ExitStatus, Error>{
        let url = format!("https://www.youtube.com/watch?v={}", id);
        let mut args = vec!["-f", "ba", "-x", "--audio-format", "mp3",
            "-o", output, "--js-runtimes", "node",
            "--js-runtimes", "deno", "--retries", "10",
            "--retry-sleep", "5"];
        args.extend(self.cookies_args());
        args.push(&url);
        Command::new(&self.path)
            .args(&args)
            .spawn()?
            .wait()
            .await
            .map_err(|e| e.into())
    }

    pub async fn auto_update() -> Result<(), Error>{
        let python3 = "python3";
        let args = vec!["-m", "pip", "install", "--user", "--upgrade",
            "--break-system-packages", "yt-dlp[default]"];
        // Async process wait (non-blocking-update-paths): the pip run can take
        // tens of seconds; a synchronous `std::process::Command::wait()` here
        // would pin a tokio worker thread for the whole time.
        let mut child = Command::new(python3)
            .args(&args)
            .spawn()
            .map_err(|e| Error::default(&e.to_string()))?;
        let status = child
            .wait()
            .await
            .map_err(|e| Error::default(&e.to_string()))?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::default("Can't update yt-dlp"))
        }
    }
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

#[tokio::test]
async fn test_e(){
    let ytdlp = Ytdlp::new("yt-dlp", "cookies.txt");
    // Old date: the "error" channel yields no parseable videos.
    let old = Utc.timestamp_opt(0, 0).unwrap();
    let salida = ytdlp.get_latest("error", old).await;
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
    let today = Utc::now();
    let salida = ytdlp.get_latest("atareao", today).await;
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

