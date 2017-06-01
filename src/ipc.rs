use serde_json::{self, Value};
use std::collections::HashMap;
use std::io::BufReader;
use std::io::prelude::*;
use std::iter::Iterator;
use std::sync::mpsc::Sender;
use super::*;

#[derive(Debug)]
pub struct PlaylistEntry {
    pub id: usize,
    pub filename: String,
    pub title: String,
    pub current: bool,
}

pub trait TypeHandler: Sized {
    fn get_value(value: Value) -> Result<Self, Error>;
    fn as_string(&self) -> String;
}

impl TypeHandler for String {
    fn get_value(value: Value) -> Result<String, Error> {
        if let Value::Object(map) = value {
            if let Value::String(ref error) = map["error"] {
                if error == "success" && map.contains_key("data") {
                    if let Value::String(ref s) = map["data"] {
                        Ok(s.to_string())
                    } else {
                        Err(Error(ErrorCode::ValueDoesNotContainString))
                    }
                } else {
                    Err(Error(ErrorCode::MpvError(error.to_string())))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedValue))
            }
        } else {
            Err(Error(ErrorCode::UnexpectedValue))
        }
    }

    fn as_string(&self) -> String {
        self.to_string()
    }
}

impl TypeHandler for bool {
    fn get_value(value: Value) -> Result<bool, Error> {
        if let Value::Object(map) = value {
            if let Value::String(ref error) = map["error"] {
                if error == "success" && map.contains_key("data") {
                    if let Value::Bool(ref b) = map["data"] {
                        Ok(*b)
                    } else {
                        Err(Error(ErrorCode::ValueDoesNotContainBool))
                    }
                } else {
                    Err(Error(ErrorCode::MpvError(error.to_string())))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedValue))
            }
        } else {
            Err(Error(ErrorCode::UnexpectedValue))
        }
    }
    fn as_string(&self) -> String {
        if *self {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }
}

impl TypeHandler for f64 {
    fn get_value(value: Value) -> Result<f64, Error> {
        if let Value::Object(map) = value {
            if let Value::String(ref error) = map["error"] {
                if error == "success" && map.contains_key("data") {
                    if let Value::Number(ref num) = map["data"] {
                        Ok(num.as_f64().unwrap())
                    } else {
                        Err(Error(ErrorCode::ValueDoesNotContainF64))
                    }
                } else {
                    Err(Error(ErrorCode::MpvError(error.to_string())))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedValue))
            }
        } else {
            Err(Error(ErrorCode::UnexpectedValue))
        }
    }

    fn as_string(&self) -> String {
        self.to_string()
    }
}

impl TypeHandler for usize {
    fn get_value(value: Value) -> Result<usize, Error> {
        if let Value::Object(map) = value {
            if let Value::String(ref error) = map["error"] {
                if error == "success" && map.contains_key("data") {
                    if let Value::Number(ref num) = map["data"] {
                        Ok(num.as_u64().unwrap() as usize)
                    } else {
                        Err(Error(ErrorCode::ValueDoesNotContainUsize))
                    }
                } else {
                    Err(Error(ErrorCode::MpvError(error.to_string())))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedValue))
            }
        } else {
            Err(Error(ErrorCode::UnexpectedValue))
        }
    }

    fn as_string(&self) -> String {
        self.to_string()
    }
}

impl TypeHandler for HashMap<String, String> {
    fn get_value(value: Value) -> Result<HashMap<String, String>, Error> {
        if let Value::Object(map) = value {
            if let Value::String(ref error) = map["error"] {
                if error == "success" && map.contains_key("data") {
                    if let Value::Object(ref inner_map) = map["data"] {
                        let mut output_map: HashMap<String, String> = HashMap::new();
                        for (ref key, ref value) in inner_map.iter() {
                            if let Value::String(ref val) = **value {
                                output_map.insert(key.to_string(), val.to_string());
                            }
                        }
                        let output_map = output_map;
                        Ok(output_map)
                    } else {
                        Err(Error(ErrorCode::ValueDoesNotContainHashMap))
                    }
                } else {
                    Err(Error(ErrorCode::MpvError(error.to_string())))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedValue))
            }
        } else {
            Err(Error(ErrorCode::UnexpectedValue))
        }
    }

    fn as_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl TypeHandler for Vec<PlaylistEntry> {
    fn get_value(value: Value) -> Result<Vec<PlaylistEntry>, Error> {
        if let Value::Object(map) = value {
            if let Value::String(ref error) = map["error"] {
                if error == "success" && map.contains_key("data") {
                    if let Value::Array(ref playlist_vec) = map["data"] {
                        let mut output: Vec<PlaylistEntry> = Vec::new();
                        for (id, entry) in playlist_vec.iter().enumerate() {
                            let mut filename: String = String::new();
                            let mut title: String = String::new();
                            let mut current: bool = false;
                            if let Value::String(ref f) = entry["filename"] {
                                filename = f.to_string();
                            }
                            if let Value::String(ref t) = entry["title"] {
                                title = t.to_string();
                            }
                            if let Value::Bool(ref b) = entry["current"] {
                                current = *b;
                            }
                            output.push(PlaylistEntry {
                                            id: id,
                                            filename: filename,
                                            title: title,
                                            current: current,
                                        });
                        }
                        let output = output;
                        Ok(output)
                    } else {
                        Err(Error(ErrorCode::ValueDoesNotContainPlaylist))
                    }
                } else {
                    Err(Error(ErrorCode::MpvError(error.to_string())))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedValue))
            }
        } else {
            Err(Error(ErrorCode::UnexpectedValue))
        }
    }

    fn as_string(&self) -> String {
        format!("{:?}", self)
    }
}

pub fn get_mpv_property<T: TypeHandler>(instance: &Mpv, property: &str) -> Result<T, Error> {
    let ipc_string = format!("{{ \"command\": [\"get_property\",\"{}\"] }}\n", property);

    match serde_json::from_str::<Value>(&send_command_sync(instance, &ipc_string)) {
        Ok(val) => T::get_value(val),
        Err(why) => Err(Error(ErrorCode::JsonParseError(why.to_string()))),
    }
}

pub fn get_mpv_property_string(instance: &Mpv, property: &str) -> Result<String, Error> {
    let ipc_string = format!("{{ \"command\": [\"get_property\",\"{}\"] }}\n", property);
    match serde_json::from_str::<Value>(&send_command_sync(instance, &ipc_string)) {
        Ok(val) => {
            if let Value::Object(map) = val {
                if let Value::String(ref error) = map["error"] {
                    if error == "success" && map.contains_key("data") {
                        match map["data"] {
                            Value::Bool(b) => Ok(b.to_string()),
                            Value::Number(ref n) => Ok(n.to_string()),
                            Value::String(ref s) => Ok(s.to_string()),
                            Value::Array(ref array) => Ok(format!("{:?}", array)),
                            Value::Object(ref map) => Ok(format!("{:?}", map)),
                            _ => Err(Error(ErrorCode::UnsupportedType)),
                        }
                    } else {
                        Err(Error(ErrorCode::MpvError(error.to_string())))
                    }
                } else {
                    Err(Error(ErrorCode::UnexpectedValue))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedValue))
            }
        }
        Err(why) => Err(Error(ErrorCode::JsonParseError(why.to_string()))),
    }
}

pub fn set_mpv_property<T: TypeHandler>(instance: &Mpv,
                                        property: &str,
                                        value: T)
                                        -> Result<(), Error> {
    let ipc_string = format!("{{ \"command\": [\"set_property\", \"{}\", {}] }}\n",
                             property,
                             value.as_string());
    match serde_json::from_str::<Value>(&send_command_sync(instance, &ipc_string)) {
        Ok(_) => Ok(()),
        Err(why) => Err(Error(ErrorCode::JsonParseError(why.to_string()))),
    }
}

pub fn run_mpv_command(instance: &Mpv, command: &str, args: &[&str]) -> Result<(), Error> {
    let mut ipc_string = format!("{{ \"command\": [\"{}\"", command);
    if args.len() > 0 {
        for arg in args {
            ipc_string.push_str(&format!(", \"{}\"", arg));
        }
    }
    ipc_string.push_str("] }\n");
    ipc_string = ipc_string;
    match serde_json::from_str::<Value>(&send_command_sync(instance, &ipc_string)) {
        Ok(feedback) => {
            if let Value::String(ref error) = feedback["error"] {
                if error == "success" {
                    Ok(())
                } else {
                    Err(Error(ErrorCode::MpvError(error.to_string())))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedResult))
            }
        }
        Err(why) => Err(Error(ErrorCode::JsonParseError(why.to_string()))),
    }
}

pub fn observe_mpv_property(instance: &Mpv, id: &usize, property: &str) -> Result<(), Error> {
    let ipc_string = format!("{{ \"command\": [\"observe_property\", {}, \"{}\"] }}\n",
                             id,
                             property);
    match serde_json::from_str::<Value>(&send_command_sync(instance, &ipc_string)) {
        Ok(feedback) => {
            if let Value::String(ref error) = feedback["error"] {
                if error == "success" {
                    Ok(())
                } else {
                    Err(Error(ErrorCode::MpvError(error.to_string())))
                }
            } else {
                Err(Error(ErrorCode::UnexpectedResult))
            }
        }
        Err(why) => Err(Error(ErrorCode::JsonParseError(why.to_string()))),
    }
}

/// #Description
///
/// Listens on socket <socket> for events and prints them in real-time to stdout.
/// This function contains an infinite-loop which keeps the application open indefinitely.
///
/// #Example
/// ```
/// listen("/tmp/mpvsocket");
/// ```
pub fn listen(instance: &Mpv, tx: &Sender<Event>) {
    let mut response = String::new();
    let mut reader = BufReader::new(&instance.0);
    reader.read_line(&mut response).unwrap();
    match serde_json::from_str::<Value>(&response) {
        Ok(e) => {
            if let Value::String(ref name) = e["event"] {
                let event: Event = match name.as_str() {
                    "shutdown" => Event::Shutdown,
                    "start-file" => Event::StartFile,
                    "file-loaded" => Event::FileLoaded,
                    "seek" => Event::Seek,
                    "playback-restart" => Event::PlaybackRestart,
                    "idle" => Event::Idle,
                    "tick" => Event::Tick,
                    "video-reconfig" => Event::VideoReconfig,
                    "audio-reconfig" => Event::AudioReconfig,
                    "tracks-changed" => Event::TracksChanged,
                    "track-switched" => Event::TrackSwitched,
                    "pause" => Event::Pause,
                    "unpause" => Event::Unpause,
                    "metadata-update" => Event::MetadataUpdate,
                    "chapter-change" => Event::ChapterChange,
                    "end-file" => Event::EndFile,
                    _ => Event::Unimplemented,
                };
                tx.send(event).unwrap();
            }
        }
        Err(why) => panic!("{}", why.to_string()),
    }
    response.clear();
}

pub fn listen_raw(instance: &Mpv, tx: &Sender<String>) {
    let mut response = String::new();
    let mut reader = BufReader::new(&instance.0);
    reader.read_line(&mut response).unwrap();
    tx.send(response.clone()).unwrap();
    response.clear();
}

fn send_command_sync(instance: &Mpv, command: &str) -> String {
    let mut stream = &instance.0;
    match stream.write_all(command.as_bytes()) {
        Err(why) => panic!("Error: Could not write to socket: {}", why),
        Ok(_) => {
            let mut response = String::new();
            {
                let mut reader = BufReader::new(stream);
                while !response.contains("\"error\":") {
                    response.clear();
                    reader.read_line(&mut response).unwrap();
                }
            }
            response
        }
    }
}
