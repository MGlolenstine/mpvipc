use env_logger;
use mpvipc::{Error as MpvError, Mpv};

#[tokio::main]
async fn main() -> Result<(), MpvError> {
    env_logger::init();

    let mpv = Mpv::connect("/tmp/mpvsocket").await?;
    let meta = mpv.get_metadata().await?;
    println!("metadata: {:?}", meta);
    let playlist = mpv.get_playlist().await?;
    println!("playlist: {:?}", playlist);
    let playback_time: f64 = mpv.get_property("playback-time").await?;
    println!("playback-time: {}", playback_time);
    Ok(())
}
