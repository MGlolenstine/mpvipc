use env_logger;
use mpvipc::{
    Error,
    Event,
    Mpv,
    MpvDataType,
};
use std::io::{self, Write};

fn seconds_to_hms(total: f64) -> String {
    let total = total as u64;
    let seconds = total % 60;
    let total = total / 60;
    let minutes = total % 60;
    let hours = total / 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let mut mpv = Mpv::connect("/tmp/mpvsocket")?;
    let mut pause = false;
    let mut playback_time = std::f64::NAN;
    let mut duration = std::f64::NAN;
    mpv.observe_property(&1, "path")?;
    mpv.observe_property(&2, "pause")?;
    mpv.observe_property(&3, "playback-time")?;
    mpv.observe_property(&4, "duration")?;
    mpv.observe_property(&5, "metadata")?;
    loop {
        let event = mpv.event_listen()?;
        match event {
            Event::PropertyChange { name, id: _, data } => {
                match name.as_ref() {
                    "path" => {
                        match data {
                            MpvDataType::String(value) => println!("\nPlaying: {}[K", value),
                            MpvDataType::Null => (),
                            _ => panic!("Wrong data type for 'path' value: {:?}", data),
                        }
                    },
                    "pause" => {
                        match data {
                            MpvDataType::Bool(value) => pause = value,
                            _ => panic!("Wrong data type for 'pause' value: {:?}", data),
                        }
                    },
                    "playback-time" => {
                        match data {
                            MpvDataType::Double(value) => playback_time = value,
                            MpvDataType::Null => (),
                            _ => panic!("Wrong data type for 'playback-time' value: {:?}", data),
                        }
                    },
                    "duration" => {
                        match data {
                            MpvDataType::Double(value) => duration = value,
                            MpvDataType::Null => (),
                            _ => panic!("Wrong data type for 'duration' value: {:?}", data),
                        }
                    },
                    "metadata" => {
                        match data {
                            MpvDataType::HashMap(value) => {
                                println!("File tags:");
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
                            },
                            MpvDataType::Null => (),
                            _ => panic!("Wrong data type for 'metadata' value: {:?}", data),
                        }
                    },
                    _ => panic!("Wrong property changed: {}", name),
                }
            },
            Event::Shutdown => return Ok(()),
            Event::Unimplemented => panic!("Unimplemented event"),
            _ => (),
        }
        print!("{}{} / {} ({:.0}%)[K\r", if pause { "(Paused) " } else { "" }, seconds_to_hms(playback_time), seconds_to_hms(duration), 100. * playback_time / duration);
        io::stdout().flush().unwrap();
    }
}
