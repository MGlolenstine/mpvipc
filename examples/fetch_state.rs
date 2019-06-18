use env_logger;
use mpvipc::{Mpv, Error as MpvError};

fn main() -> Result<(), MpvError> {
    env_logger::init();

    let mpv = Mpv::connect("/tmp/mpvsocket")?;
    let meta = mpv.get_metadata()?;
    println!("metadata: {:?}", meta);
    let playlist = mpv.get_playlist()?;
    println!("playlist: {:?}", playlist);
    let playback_time: f64 = mpv.get_property("playback-time")?;
    println!("playback-time: {}", playback_time);
    Ok(())
}
