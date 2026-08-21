use sqlx::SqlitePool;
use tracing::{
    info,
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
    let ytvideos = ytdlp.get_latest(&channel.url, last).await?;
    info!("Getting {} videos", ytvideos.len());
    for ytvideo in ytvideos{
        info!("Processing: {}", &ytvideo.title);
        match process_episode(pool, channel, &ytvideo, ytdlp, folder).await{
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

    if !ytdlp.download(&ytvideo.id, &filename).await?.success(){
        Err(Error::default(&format!("Cant download {filename}")))?
    }
    let delay = rand::thread_rng().gen_range(20..=40);
    info!("Pausing {delay} seconds before next download");
    sleep(Duration::from_secs(delay)).await;
    let title = &ytvideo.title;
    let description = &ytvideo.description;
    let yt_id = &ytvideo.id;
    let webpage_url = &ytvideo.webpage_url;
    let duration = &ytvideo.duration_string;
    info!("{}", &ytvideo.upload_date);
    let published_at = get_published_at(ytvideo);
    let _ = filetime::set_file_mtime(
        &filename,
        filetime::FileTime::from_unix_time(
            published_at.timestamp(), 0)
    );
    let image = &ytvideo.thumbnail;
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
    // Fallback so a malformed/youtube edge-case date never panics the worker.
    error!("Cant parse publish date from {:?}, using now()", &ytvideo.upload_date);
    Utc::now()
}

