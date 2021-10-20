use mpvipc::ipc::{send_flat_command};
use mpvipc::{Error, Mpv};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let mpv = Mpv::connect("/tmp/mpvsocket").await?;
    mpv.playlist_add("small.mp4",mpvipc::PlaylistAddTypeOptions::File,mpvipc::PlaylistAddOptions::Append).await.unwrap();
    mpv.playlist_play_id(0).await.unwrap();
    tokio::time::sleep(Duration::from_millis(1000)).await;
    send_flat_command(&mpv, "seek 50.123 absolute\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(1000)).await;
    send_flat_command(&mpv, "seek 50.123 absolute\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(1000)).await;
    send_flat_command(&mpv, "seek 50.123 absolute\n").await.unwrap();
    Ok(())
}
