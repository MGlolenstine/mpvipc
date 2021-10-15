use env_logger;
use mpvipc::{Error, Event, Mpv, MpvDataType, Property};
use std::io::{self, Write};

fn seconds_to_hms(total: f64) -> String {
    let total = total as u64;
    let seconds = total % 60;
    let total = total / 60;
    let minutes = total % 60;
    let hours = total / 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let mut mpv = Mpv::connect("/tmp/mpvsocket").await?;
    let mut pause = false;
    let mut playback_time = std::f64::NAN;
    let mut duration = std::f64::NAN;
    mpv.observe_property(&1, "path").await?;
    mpv.observe_property(&2, "pause").await?;
    mpv.observe_property(&3, "playback-time").await?;
    mpv.observe_property(&4, "duration").await?;
    mpv.observe_property(&5, "metadata").await?;
    loop {
        let event = mpv.event_listen().await?;
        match event {
            Event::PropertyChange { id: _, property } => match property {
                Property::Path(Some(value)) => println!("\nPlaying: {}[K", value),
                Property::Path(None) => (),
                Property::Pause(value) => pause = value,
                Property::PlaybackTime(Some(value)) => playback_time = value,
                Property::PlaybackTime(None) => playback_time = std::f64::NAN,
                Property::Duration(Some(value)) => duration = value,
                Property::Duration(None) => duration = std::f64::NAN,
                Property::Metadata(Some(value)) => {
                    println!("File tags:[K");
                    if let Some(MpvDataType::String(value)) = value.get("ARTIST") {
                        println!(" Artist: {}[K", value);
                    }
                    if let Some(MpvDataType::String(value)) = value.get("ALBUM") {
                        println!(" Album: {}[K", value);
                    }
                    if let Some(MpvDataType::String(value)) = value.get("TITLE") {
                        println!(" Title: {}[K", value);
                    }
                    if let Some(MpvDataType::String(value)) = value.get("TRACK") {
                        println!(" Track: {}[K", value);
                    }
                }
                Property::Metadata(None) => (),
                Property::Unknown { name: _, data: _ } => (),
            },
            Event::Shutdown => return Ok(()),
            Event::Unimplemented => panic!("Unimplemented event"),
            _ => (),
        }
        print!(
            "{}{} / {} ({:.0}%)[K\r",
            if pause { "(Paused) " } else { "" },
            seconds_to_hms(playback_time),
            seconds_to_hms(duration),
            100. * playback_time / duration
        );
        io::stdout().flush().unwrap();
    }
}
