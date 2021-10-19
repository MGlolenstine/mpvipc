pub mod ipc;

use async_trait::async_trait;

use ipc::*;
use log::{debug, trace, warn};
use serde_json::{Deserializer, Value};
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::io::{BufReader, Read};
use std::os::unix::net::UnixStream;
// use tokio::sync::broadcast::{Receiver, Sender};
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Shutdown,
    StartFile,
    EndFile,
    FileLoaded,
    TracksChanged,
    TrackSwitched,
    Idle,
    Pause,
    Unpause,
    Tick,
    VideoReconfig,
    AudioReconfig,
    MetadataUpdate,
    Seek,
    PlaybackRestart,
    PropertyChange { id: isize, property: Property },
    ChapterChange,
    Unimplemented,
}

impl From<MpvEvent> for Event {
    fn from(event: MpvEvent) -> Self {
        match event.event.as_str() {
            "pause" => Self::Pause,
            "unpause" => Self::Unpause,
            "shutdown" => Self::Shutdown,
            "file_loaded" => Self::FileLoaded,
            "tracks_changed" => Self::TracksChanged,
            "track_switched" => Self::TrackSwitched,
            "idle" => Self::Idle,
            "seek" => Self::Seek,
            "video_reconfig" => Self::VideoReconfig,
            "audio_reconfig" => Self::AudioReconfig,
            _ => {
                trace!("Event {:#?} hasn't been implemented yet!", event.event);
                Self::Unimplemented
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Property {
    Path(Option<String>),
    Pause(bool),
    PlaybackTime(Option<f64>),
    Duration(Option<f64>),
    Metadata(Option<HashMap<String, MpvDataType>>),
    Unknown { name: String, data: MpvDataType },
}

pub enum MpvCommand {
    LoadFile {
        file: String,
        option: PlaylistAddOptions,
    },
    LoadList {
        file: String,
        option: PlaylistAddOptions,
    },
    PlaylistClear,
    PlaylistMove {
        from: usize,
        to: usize,
    },
    PlaylistNext,
    PlaylistPrev,
    PlaylistRemove(usize),
    PlaylistShuffle,
    Quit,
    Seek {
        seconds: f64,
        option: SeekOptions,
    },
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MpvDataType {
    Array(Vec<MpvDataType>),
    Bool(bool),
    Double(f64),
    HashMap(HashMap<String, MpvDataType>),
    Null,
    Playlist(Playlist),
    String(String),
    Usize(usize),
}

pub enum NumberChangeOptions {
    Absolute,
    Increase,
    Decrease,
}

pub enum PlaylistAddOptions {
    Replace,
    Append,
}

pub enum PlaylistAddTypeOptions {
    File,
    Playlist,
}

pub enum SeekOptions {
    Relative,
    Absolute,
    RelativePercent,
    AbsolutePercent,
}

pub enum Switch {
    On,
    Off,
    Toggle,
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    MpvError(String),
    JsonParseError(String),
    ConnectError(String),
    JsonContainsUnexptectedType,
    UnexpectedResult,
    UnexpectedValue,
    UnsupportedType,
    ValueDoesNotContainBool,
    ValueDoesNotContainF64,
    ValueDoesNotContainHashMap,
    ValueDoesNotContainPlaylist,
    ValueDoesNotContainString,
    ValueDoesNotContainUsize,
}

pub struct Mpv {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
    name: String,
    pub event_receiver: Option<Receiver<Event>>,
    response_receiver: Mutex<Receiver<Data>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist(pub Vec<PlaylistEntry>);
#[derive(Debug, Clone)]
pub struct Error(pub ErrorCode);

impl Drop for Mpv {
    fn drop(&mut self) {
        self.disconnect();
    }
}

impl fmt::Debug for Mpv {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("Mpv").field(&self.name).finish()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Data {
    data: Value,
    request_id: u32,
    error: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Response {
    request_id: u32,
    error: String,
}

impl Into<Data> for Response {
    fn into(self) -> Data {
        Data {
            data: serde_json::Value::Null,
            request_id: self.request_id,
            error: self.error,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct MpvEvent {
    event: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum MpvMessage {
    Event(MpvEvent),
    Data(Data),
    GenericResponse(Response),
    Other(serde_json::Value),
}

//?  Can't really be implemented due to listeners. Would have to start a new loop with new senders and receivers.
// impl Clone for Mpv {
//     fn clone(&self) -> Self {
//         let stream = self.stream.try_clone().expect("cloning UnixStream");
//         let cloned_stream = stream.try_clone().expect("cloning UnixStream");
//         Mpv {
//             stream,
//             reader: BufReader::new(cloned_stream),
//             name: self.name.clone(),
//         }
//     }

//     fn clone_from(&mut self, source: &Self) {
//         let stream = source.stream.try_clone().expect("cloning UnixStream");
//         let cloned_stream = stream.try_clone().expect("cloning UnixStream");
//         *self = Mpv {
//             stream,
//             reader: BufReader::new(cloned_stream),
//             name: source.name.clone(),
//         }
//     }
// }

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl std::error::Error for Error {}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::ConnectError(ref msg) => f.write_str(&format!("ConnectError: {}", msg)),
            ErrorCode::JsonParseError(ref msg) => f.write_str(&format!("JsonParseError: {}", msg)),
            ErrorCode::MpvError(ref msg) => f.write_str(&format!("MpvError: {}", msg)),
            ErrorCode::JsonContainsUnexptectedType => {
                f.write_str("Mpv sent a value with an unexpected type")
            }
            ErrorCode::UnexpectedResult => f.write_str("Unexpected result received"),
            ErrorCode::UnexpectedValue => f.write_str("Unexpected value received"),
            ErrorCode::UnsupportedType => f.write_str("Unsupported type received"),
            ErrorCode::ValueDoesNotContainBool => {
                f.write_str("The received value is not of type \'std::bool\'")
            }
            ErrorCode::ValueDoesNotContainF64 => {
                f.write_str("The received value is not of type \'std::f64\'")
            }
            ErrorCode::ValueDoesNotContainHashMap => {
                f.write_str("The received value is not of type \'std::collections::HashMap\'")
            }
            ErrorCode::ValueDoesNotContainPlaylist => {
                f.write_str("The received value is not of type \'mpvipc::Playlist\'")
            }
            ErrorCode::ValueDoesNotContainString => {
                f.write_str("The received value is not of type \'std::string::String\'")
            }
            ErrorCode::ValueDoesNotContainUsize => {
                f.write_str("The received value is not of type \'std::usize\'")
            }
        }
    }
}

#[async_trait]
pub trait GetPropertyTypeHandler: Sized {
    async fn get_property_generic(instance: &Mpv, property: &str) -> Result<Self, Error>;
}

#[async_trait]
impl GetPropertyTypeHandler for bool {
    async fn get_property_generic(instance: &Mpv, property: &str) -> Result<bool, Error> {
        get_mpv_property::<bool>(instance, property).await
    }
}

#[async_trait]
impl GetPropertyTypeHandler for String {
    async fn get_property_generic(instance: &Mpv, property: &str) -> Result<String, Error> {
        get_mpv_property::<String>(instance, property).await
    }
}

#[async_trait]
impl GetPropertyTypeHandler for f64 {
    async fn get_property_generic(instance: &Mpv, property: &str) -> Result<f64, Error> {
        get_mpv_property::<f64>(instance, property).await
    }
}

#[async_trait]
impl GetPropertyTypeHandler for usize {
    async fn get_property_generic(instance: &Mpv, property: &str) -> Result<usize, Error> {
        get_mpv_property::<usize>(instance, property).await
    }
}

#[async_trait]
impl GetPropertyTypeHandler for Vec<PlaylistEntry> {
    async fn get_property_generic(
        instance: &Mpv,
        property: &str,
    ) -> Result<Vec<PlaylistEntry>, Error> {
        get_mpv_property::<Vec<PlaylistEntry>>(instance, property).await
    }
}

#[async_trait]
impl GetPropertyTypeHandler for HashMap<String, MpvDataType> {
    async fn get_property_generic(
        instance: &Mpv,
        property: &str,
    ) -> Result<HashMap<String, MpvDataType>, Error> {
        get_mpv_property::<HashMap<String, MpvDataType>>(instance, property).await
    }
}

#[async_trait]
pub trait SetPropertyTypeHandler<T> {
    async fn set_property_generic(instance: &Mpv, property: &str, value: T) -> Result<(), Error>;
}

#[async_trait]
impl SetPropertyTypeHandler<bool> for bool {
    async fn set_property_generic(
        instance: &Mpv,
        property: &str,
        value: bool,
    ) -> Result<(), Error> {
        set_mpv_property::<bool>(instance, property, value).await
    }
}

#[async_trait]
impl SetPropertyTypeHandler<String> for String {
    async fn set_property_generic(
        instance: &Mpv,
        property: &str,
        value: String,
    ) -> Result<(), Error> {
        set_mpv_property::<String>(instance, property, value).await
    }
}

#[async_trait]
impl SetPropertyTypeHandler<f64> for f64 {
    async fn set_property_generic(instance: &Mpv, property: &str, value: f64) -> Result<(), Error> {
        set_mpv_property::<f64>(instance, property, value).await
    }
}

#[async_trait]
impl SetPropertyTypeHandler<usize> for usize {
    async fn set_property_generic(
        instance: &Mpv,
        property: &str,
        value: usize,
    ) -> Result<(), Error> {
        set_mpv_property::<usize>(instance, property, value).await
    }
}

impl Mpv {
    async fn start_listeners(eventtx: Sender<Event>, responsetx: Sender<Data>, stream: UnixStream) {
        let event_clone = eventtx.clone();
        let response_clone = responsetx.clone();
        // tokio::task::spawn_blocking(move ||{
        std::thread::spawn(move || {
            let reader = BufReader::new(stream);
            for item in Deserializer::from_reader(reader).into_iter::<MpvMessage>() {
                match item {
                    Ok(MpvMessage::Data(a)) => {
                        debug!("Data: {:#?}", a);
                        response_clone.blocking_send(a).unwrap();
                    }
                    Ok(MpvMessage::GenericResponse(a)) => {
                        debug!("Generic Response: {:#?}", a);
                        response_clone.blocking_send(a.into()).unwrap();
                        // responsetx.send(a.into()).await.unwrap();
                    }
                    Ok(MpvMessage::Event(e)) => {
                        debug!("Event: {:#?}", e);
                        // eventtx.send(e.into()).await.unwrap();
                        event_clone.blocking_send(e.into()).unwrap();
                    }
                    _ => {
                        warn!("Unhandled message: {:#?}", item);
                    }
                }
            }
        });
    }

    pub async fn connect(socket: &str) -> Result<Mpv, Error> {
        match UnixStream::connect(socket) {
            Ok(stream) => {
                let cloned_stream = stream.try_clone().expect("cloning UnixStream");
                let (eventtx, eventrx) = tokio::sync::mpsc::channel::<Event>(8);
                let (responsetx, responserx) = tokio::sync::mpsc::channel::<Data>(8);

                Mpv::start_listeners(
                    eventtx,
                    responsetx,
                    stream.try_clone().expect("cloning UnixStream"),
                )
                .await;
                Ok(Mpv {
                    stream,
                    reader: BufReader::new(cloned_stream),
                    name: String::from(socket),
                    event_receiver: Some(eventrx),
                    response_receiver: Mutex::new(responserx),
                })
            }
            Err(internal_error) => Err(Error(ErrorCode::ConnectError(internal_error.to_string()))),
        }
    }

    pub fn disconnect(&self) {
        let mut stream = &self.stream;
        stream
            .shutdown(std::net::Shutdown::Both)
            .expect("socket disconnect");
        let mut buffer = [0; 32];
        for _ in 0..stream.bytes().count() {
            stream.read_exact(&mut buffer[..]).unwrap();
        }
    }

    pub fn get_stream_ref(&self) -> &UnixStream {
        &self.stream
    }

    pub async fn get_metadata(&self) -> Result<HashMap<String, MpvDataType>, Error> {
        match get_mpv_property(self, "metadata").await {
            Ok(map) => Ok(map),
            Err(err) => Err(err),
        }
    }

    pub async fn get_playlist(&self) -> Result<Playlist, Error> {
        match get_mpv_property::<Vec<PlaylistEntry>>(self, "playlist").await {
            Ok(entries) => Ok(Playlist(entries)),
            Err(msg) => Err(msg),
        }
    }

    /// # Description
    ///
    /// Retrieves the property value from mpv.
    ///
    /// ## Supported types
    /// - String
    /// - bool
    /// - HashMap<String, String> (e.g. for the 'metadata' property)
    /// - Vec<PlaylistEntry> (for the 'playlist' property)
    /// - usize
    /// - f64
    ///
    /// ## Input arguments
    ///
    /// - **property** defines the mpv property that should be retrieved
    ///
    /// # Example
    /// ```
    /// use mpvipc::{Mpv, Error};
    /// fn main() -> Result<(), Error> {
    ///     let mpv = Mpv::connect("/tmp/mpvsocket")?;
    ///     let paused: bool = mpv.get_property("pause")?;
    ///     let title: String = mpv.get_property("media-title")?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_property<T: GetPropertyTypeHandler>(
        &self,
        property: &str,
    ) -> Result<T, Error> {
        T::get_property_generic(self, property).await
    }

    /// # Description
    ///
    /// Retrieves the property value from mpv.
    /// The result is always of type String, regardless of the type of the value of the mpv property
    ///
    /// ## Input arguments
    ///
    /// - **property** defines the mpv property that should be retrieved
    ///
    /// # Example
    ///
    /// ```
    /// use mpvipc::{Mpv, Error};
    /// fn main() -> Result<(), Error> {
    ///     let mpv = Mpv::connect("/tmp/mpvsocket")?;
    ///     let title = mpv.get_property_string("media-title")?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_property_string(&self, property: &str) -> Result<String, Error> {
        get_mpv_property_string(self, property).await
    }

    pub async fn kill(&self) -> Result<(), Error> {
        self.run_command(MpvCommand::Quit).await
    }

    /// # Description
    ///
    /// Waits until an mpv event occurs and returns the Event.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut mpv = Mpv::connect("/tmp/mpvsocket")?;
    /// loop {
    ///     let event = mpv.event_listen()?;
    ///     println!("{:?}", event);
    /// }
    /// ```
    // pub async fn event_listen(&mut self) -> Result<Event, Error> {
    //     listen(self).await
    // }

    pub fn event_listen_raw(&mut self) -> String {
        listen_raw(self)
    }

    pub async fn next(&self) -> Result<(), Error> {
        self.run_command(MpvCommand::PlaylistNext).await
    }

    pub async fn observe_property(&self, id: &isize, property: &str) -> Result<(), Error> {
        observe_mpv_property(self, id, property).await
    }

    pub async fn pause(&self) -> Result<(), Error> {
        set_mpv_property(self, "pause", true).await
    }

    pub async fn prev(&self) -> Result<(), Error> {
        self.run_command(MpvCommand::PlaylistPrev).await
    }

    pub async fn restart(&self) -> Result<(), Error> {
        self.run_command(MpvCommand::Seek {
            seconds: 0f64,
            option: SeekOptions::Absolute,
        })
        .await
    }

    /// # Description
    ///
    /// Runs mpv commands. The arguments are passed as a String-Vector reference:
    ///
    /// ## Input arguments
    ///
    /// - **command**   defines the mpv command that should be executed
    /// - **args**      a slice of &str's which define the arguments
    ///
    /// # Example
    /// ```
    /// use mpvipc::{Mpv, Error};
    /// fn main() -> Result<(), Error> {
    ///     let mpv = Mpv::connect("/tmp/mpvsocket")?;
    ///
    ///     //Run command 'playlist-shuffle' which takes no arguments
    ///     mpv.run_command(MpvCommand::PlaylistShuffle)?;
    ///
    ///     //Run command 'seek' which in this case takes two arguments
    ///     mpv.run_command(MpvCommand::Seek {
    ///         seconds: 0f64,
    ///         option: SeekOptions::Absolute,
    ///     })?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_command(&self, command: MpvCommand) -> Result<(), Error> {
        match command {
            MpvCommand::LoadFile { file, option } => {
                run_mpv_command(
                    self,
                    "loadfile",
                    &[
                        file.as_ref(),
                        match option {
                            PlaylistAddOptions::Append => "append",
                            PlaylistAddOptions::Replace => "replace",
                        },
                    ],
                )
                .await
            }
            MpvCommand::LoadList { file, option } => {
                run_mpv_command(
                    self,
                    "loadlist",
                    &[
                        file.as_ref(),
                        match option {
                            PlaylistAddOptions::Append => "append",
                            PlaylistAddOptions::Replace => "replace",
                        },
                    ],
                )
                .await
            }
            MpvCommand::PlaylistClear => run_mpv_command(self, "playlist-clear", &[]).await,
            MpvCommand::PlaylistMove { from, to } => {
                run_mpv_command(self, "playlist-move", &[&from.to_string(), &to.to_string()]).await
            }
            MpvCommand::PlaylistNext => run_mpv_command(self, "playlist-next", &[]).await,
            MpvCommand::PlaylistPrev => run_mpv_command(self, "playlist-prev", &[]).await,
            MpvCommand::PlaylistRemove(id) => {
                run_mpv_command(self, "playlist-remove", &[&id.to_string()]).await
            }
            MpvCommand::PlaylistShuffle => run_mpv_command(self, "playlist-shuffle", &[]).await,
            MpvCommand::Quit => run_mpv_command(self, "quit", &[]).await,
            MpvCommand::Seek { seconds, option } => {
                run_mpv_command(
                    self,
                    "seek",
                    &[
                        &seconds.to_string(),
                        match option {
                            SeekOptions::Absolute => "absolute",
                            SeekOptions::Relative => "relative",
                            SeekOptions::AbsolutePercent => "absolute-percent",
                            SeekOptions::RelativePercent => "relative-percent",
                        },
                    ],
                )
                .await
            }
            MpvCommand::Stop => run_mpv_command(self, "stop", &[]).await,
        }
    }

    /// Run a custom command.
    /// This should only be used if the desired command is not implemented
    /// with [MpvCommand].
    pub async fn run_command_raw(&self, command: &str, args: &[&str]) -> Result<(), Error> {
        run_mpv_command(self, command, args).await
    }

    pub async fn playlist_add(
        &self,
        file: &str,
        file_type: PlaylistAddTypeOptions,
        option: PlaylistAddOptions,
    ) -> Result<(), Error> {
        match file_type {
            PlaylistAddTypeOptions::File => {
                self.run_command(MpvCommand::LoadFile {
                    file: file.to_string(),
                    option,
                })
                .await
            }

            PlaylistAddTypeOptions::Playlist => {
                self.run_command(MpvCommand::LoadList {
                    file: file.to_string(),
                    option,
                })
                .await
            }
        }
    }

    pub async fn playlist_clear(&self) -> Result<(), Error> {
        self.run_command(MpvCommand::PlaylistClear).await
    }

    pub async fn playlist_move_id(&self, from: usize, to: usize) -> Result<(), Error> {
        self.run_command(MpvCommand::PlaylistMove { from, to })
            .await
    }

    pub async fn playlist_play_id(&self, id: usize) -> Result<(), Error> {
        set_mpv_property(self, "playlist-pos", id).await
    }

    pub async fn playlist_play_next(&self, id: usize) -> Result<(), Error> {
        match get_mpv_property::<usize>(self, "playlist-pos").await {
            Ok(current_id) => {
                self.run_command(MpvCommand::PlaylistMove {
                    from: id,
                    to: current_id + 1,
                })
                .await
            }
            Err(msg) => Err(msg),
        }
    }

    pub async fn playlist_remove_id(&self, id: usize) -> Result<(), Error> {
        self.run_command(MpvCommand::PlaylistRemove(id)).await
    }

    pub async fn playlist_shuffle(&self) -> Result<(), Error> {
        self.run_command(MpvCommand::PlaylistShuffle).await
    }

    pub async fn seek(&self, seconds: f64, option: SeekOptions) -> Result<(), Error> {
        self.run_command(MpvCommand::Seek { seconds, option }).await
    }

    pub async fn set_loop_file(&self, option: Switch) -> Result<(), Error> {
        let mut enabled = false;
        match option {
            Switch::On => enabled = true,
            Switch::Off => {}
            Switch::Toggle => match get_mpv_property_string(self, "loop-file").await {
                Ok(value) => match value.as_ref() {
                    "false" => {
                        enabled = true;
                    }
                    _ => {
                        enabled = false;
                    }
                },
                Err(msg) => return Err(msg),
            },
        }
        set_mpv_property(self, "loop-file", enabled).await
    }

    pub async fn set_loop_playlist(&self, option: Switch) -> Result<(), Error> {
        let mut enabled = false;
        match option {
            Switch::On => enabled = true,
            Switch::Off => {}
            Switch::Toggle => match get_mpv_property_string(self, "loop-playlist").await {
                Ok(value) => match value.as_ref() {
                    "false" => {
                        enabled = true;
                    }
                    _ => {
                        enabled = false;
                    }
                },
                Err(msg) => return Err(msg),
            },
        }
        set_mpv_property(self, "loop-playlist", enabled).await
    }

    pub async fn set_mute(&self, option: Switch) -> Result<(), Error> {
        let mut enabled = false;
        match option {
            Switch::On => enabled = true,
            Switch::Off => {}
            Switch::Toggle => match get_mpv_property::<bool>(self, "mute").await {
                Ok(value) => {
                    enabled = !value;
                }
                Err(msg) => return Err(msg),
            },
        }
        set_mpv_property(self, "mute", enabled).await
    }

    /// # Description
    ///
    /// Sets the mpv property _<property>_ to _<value>_.
    ///
    /// ## Supported types
    /// - String
    /// - bool
    /// - f64
    /// - usize
    ///
    /// ## Input arguments
    ///
    /// - **property** defines the mpv property that should be retrieved
    /// - **value** defines the value of the given mpv property _<property>_
    ///
    /// # Example
    /// ```
    /// use mpvipc::{Mpv, Error};
    /// fn main() -> Result<(), Error> {
    ///     let mpv = Mpv::connect("/tmp/mpvsocket")?;
    ///     mpv.set_property("pause", true)?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_property<T: SetPropertyTypeHandler<T>>(
        &self,
        property: &str,
        value: T,
    ) -> Result<(), Error> {
        T::set_property_generic(self, property, value).await
    }

    pub async fn set_speed(
        &self,
        input_speed: f64,
        option: NumberChangeOptions,
    ) -> Result<(), Error> {
        match get_mpv_property::<f64>(self, "speed").await {
            Ok(speed) => match option {
                NumberChangeOptions::Increase => {
                    set_mpv_property(self, "speed", speed + input_speed).await
                }

                NumberChangeOptions::Decrease => {
                    set_mpv_property(self, "speed", speed - input_speed).await
                }

                NumberChangeOptions::Absolute => set_mpv_property(self, "speed", input_speed).await,
            },
            Err(msg) => Err(msg),
        }
    }

    pub async fn set_volume(
        &self,
        input_volume: f64,
        option: NumberChangeOptions,
    ) -> Result<(), Error> {
        match get_mpv_property::<f64>(self, "volume").await {
            Ok(volume) => match option {
                NumberChangeOptions::Increase => {
                    set_mpv_property(self, "volume", volume + input_volume).await
                }

                NumberChangeOptions::Decrease => {
                    set_mpv_property(self, "volume", volume - input_volume).await
                }

                NumberChangeOptions::Absolute => {
                    set_mpv_property(self, "volume", input_volume).await
                }
            },
            Err(msg) => Err(msg),
        }
    }

    pub async fn stop(&self) -> Result<(), Error> {
        self.run_command(MpvCommand::Stop).await
    }

    pub async fn toggle(&self) -> Result<(), Error> {
        match get_mpv_property::<bool>(self, "pause").await {
            Ok(paused) => set_mpv_property(self, "pause", !paused).await,
            Err(msg) => Err(msg),
        }
    }
}
