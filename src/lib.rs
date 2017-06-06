extern crate serde;
extern crate serde_json;

pub mod ipc;

use ipc::*;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum Event {
    Shutdown,
    // LogMessage {
    //     prefix: &'static str,
    //     level: &'static str,
    //     text: &'static str,
    //     log_level: LogLevel,
    // },
    // GetPropertyReply {
    //     name: &'static str,
    //     result: Result<Format<'a>>,
    //     reply_userdata: u32,
    // },
    // SetPropertyReply(Result<()>, u32),
    // CommandReply(Result<()>, u32),
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
    PropertyChange {
        name: String,
        id: usize,
        data: MpvDataType,
    },
    ChapterChange,
    Unimplemented,
}

#[derive(Debug)]
pub enum MpvDataType {
    Bool(bool),
    String(String),
    Double(f64),
    Usize(usize),
    HashMap(HashMap<String, MpvDataType>),
    Array(Vec<MpvDataType>),
    Playlist(Playlist),
}

pub enum NumberChangeOptions {
    Absolute,
    Increase,
    Decrease,
}

pub enum PlaylistAddOptions {
    Replace,
    Append,
    AppendPlay,
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

#[derive(Debug)]
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

pub struct Mpv(UnixStream);
#[derive(Debug)]
pub struct Playlist(pub Vec<PlaylistEntry>);
#[derive(Debug)]
pub struct Error(pub ErrorCode);

impl Drop for Mpv {
    fn drop(&mut self) {
        self.0
            .shutdown(std::net::Shutdown::Both)
            .expect("stream shutdown");
    }
}

impl Clone for Mpv {
    fn clone(&self) -> Self {
        Mpv(self.0.try_clone().expect("cloning UnixStream"))
    }

    fn clone_from(&mut self, source: &Self) {
        *self = Mpv(source.0.try_clone().expect("cloning UnixStream"));
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::ConnectError(ref msg) => f.write_str(&format!("ConnectError: {}", msg)),
            ErrorCode::JsonParseError(ref msg) => f.write_str(&format!("JsonParseError: {}", msg)),
            ErrorCode::MpvError(ref msg) => {
                f.write_str(&format!("mpv returned an error value: {}", msg))
            }
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

pub trait GetPropertyTypeHandler: Sized {
    fn get_property_generic(instance: &Mpv, property: &str) -> Result<Self, Error>;
}

impl GetPropertyTypeHandler for bool {
    fn get_property_generic(instance: &Mpv, property: &str) -> Result<bool, Error> {
        get_mpv_property::<bool>(instance, property)
    }
}

impl GetPropertyTypeHandler for String {
    fn get_property_generic(instance: &Mpv, property: &str) -> Result<String, Error> {
        get_mpv_property::<String>(instance, property)
    }
}

impl GetPropertyTypeHandler for f64 {
    fn get_property_generic(instance: &Mpv, property: &str) -> Result<f64, Error> {
        get_mpv_property::<f64>(instance, property)
    }
}

impl GetPropertyTypeHandler for usize {
    fn get_property_generic(instance: &Mpv, property: &str) -> Result<usize, Error> {
        get_mpv_property::<usize>(instance, property)
    }
}

impl GetPropertyTypeHandler for Vec<PlaylistEntry> {
    fn get_property_generic(instance: &Mpv, property: &str) -> Result<Vec<PlaylistEntry>, Error> {
        get_mpv_property::<Vec<PlaylistEntry>>(instance, property)
    }
}

impl GetPropertyTypeHandler for HashMap<String, MpvDataType> {
    fn get_property_generic(instance: &Mpv,
                            property: &str)
                            -> Result<HashMap<String, MpvDataType>, Error> {
        get_mpv_property::<HashMap<String, MpvDataType>>(instance, property)
    }
}

pub trait SetPropertyTypeHandler<T> {
    fn set_property_generic(instance: &Mpv, property: &str, value: T) -> Result<(), Error>;
}

impl SetPropertyTypeHandler<bool> for bool {
    fn set_property_generic(instance: &Mpv, property: &str, value: bool) -> Result<(), Error> {
        set_mpv_property::<bool>(instance, property, value)
    }
}

impl SetPropertyTypeHandler<String> for String {
    fn set_property_generic(instance: &Mpv, property: &str, value: String) -> Result<(), Error> {
        set_mpv_property::<String>(instance, property, value)
    }
}

impl SetPropertyTypeHandler<f64> for f64 {
    fn set_property_generic(instance: &Mpv, property: &str, value: f64) -> Result<(), Error> {
        set_mpv_property::<f64>(instance, property, value)
    }
}

impl SetPropertyTypeHandler<usize> for usize {
    fn set_property_generic(instance: &Mpv, property: &str, value: usize) -> Result<(), Error> {
        set_mpv_property::<usize>(instance, property, value)
    }
}

impl Mpv {
    pub fn connect(socket: &str) -> Result<Mpv, Error> {
        match UnixStream::connect(socket) {
            Ok(stream) => Ok(Mpv(stream)),
            Err(internal_error) => Err(Error(ErrorCode::ConnectError(internal_error.to_string()))),
        }
    }

    pub fn get_metadata(&self) -> Result<HashMap<String, MpvDataType>, Error> {
        match get_mpv_property(self, "metadata") {
            Ok(map) => Ok(map),
            Err(err) => Err(err),
        }
    }

    pub fn get_playlist(&self) -> Result<Playlist, Error> {
        match get_mpv_property::<Vec<PlaylistEntry>>(self, "playlist") {
            Ok(entries) => Ok(Playlist(entries)),
            Err(msg) => Err(msg),
        }
    }

    /// #Description
    ///
    /// Retrieves the property value from mpv.
    ///
    /// ##Supported types
    /// - String
    /// - bool
    /// - HashMap<String, String> (e.g. for the 'metadata' property)
    /// - Vec<PlaylistEntry> (for the 'playlist' property)
    /// - usize
    /// - f64
    ///
    /// ##Input arguments
    ///
    /// - **property** defines the mpv property that should be retrieved
    ///
    /// #Example
    /// ```
    /// let mpv = Mpv::connect("/tmp/mpvsocket").unwrap();
    /// let paused: bool = mpv.get_property("pause").unwrap();
    /// let title: String = mpv.get_property("media-title").unwrap();
    /// ```
    pub fn get_property<T: GetPropertyTypeHandler>(&self, property: &str) -> Result<T, Error> {
        T::get_property_generic(self, property)
    }

    /// #Description
    ///
    /// Retrieves the property value from mpv.
    /// The result is always of type String, regardless of the type of the value of the mpv property
    ///
    /// ##Input arguments
    ///
    /// - **property** defines the mpv property that should be retrieved
    ///
    /// #Example
    ///
    /// ```
    /// let mpv = Mpv::connect("/tmp/mpvsocket").unwrap();
    /// let title = mpv.get_property_string("media-title").unwrap();
    /// ```
    pub fn get_property_string(&self, property: &str) -> Result<String, Error> {
        get_mpv_property_string(self, property)
    }

    pub fn kill(&self) -> Result<(), Error> {
        run_mpv_command(self, "quit", &[])
    }

    /// #Description
    ///
    /// Listens for mpv events and triggers the channel once an event has been received.
    ///
    /// ##Input arguments
    ///
    /// - **tx** A reference to the sender halve of the channel
    ///
    /// #Example
    ///
    /// ```
    /// let (tx, rx) = std::sync::mpsc::channel();
    /// loop {
    ///     mpv.event_listen(&tx);
    ///     let event = rx.recv().unwrap();
    ///     println!("{:?}", event);
    /// }
    /// ```
    pub fn event_listen(&self, tx: &Sender<Event>) -> Result<(), Error> {
        listen(self, tx)
    }

    pub fn event_listen_raw(&self, tx: &Sender<String>) {
        listen_raw(self, tx);
    }

    pub fn next(&self) -> Result<(), Error> {
        run_mpv_command(self, "playlist-next", &[])
    }

    pub fn observe_property(&self, id: &usize, property: &str) -> Result<(), Error> {
        observe_mpv_property(self, id, property)
    }

    pub fn pause(&self) -> Result<(), Error> {
        set_mpv_property(self, "pause", true)
    }

    pub fn prev(&self) -> Result<(), Error> {
        run_mpv_command(self, "playlist-prev", &[])
    }

    pub fn restart(&self) -> Result<(), Error> {
        run_mpv_command(self, "seek", &["0", "absolute"])
    }

    /// #Description
    ///
    /// Runs mpv commands. The arguments are passed as a String-Vector reference:
    ///
    /// ##Input arguments
    ///
    /// - **command**   defines the mpv command that should be executed
    /// - **args**      a slice of &str's which define the arguments
    ///
    /// #Example
    /// ```
    /// let mpv = Mpv::connect("/tmp/mpvsocket").unwrap();
    ///
    /// //Run command 'playlist-shuffle' which takes no arguments
    /// mpv.run_command("playlist-shuffle", &[]);
    ///
    /// //Run command 'seek' which in this case takes two arguments
    /// mpv.run_command("seek", &["0", "absolute"]);
    /// ```
    pub fn run_command(&self, command: &str, args: &[&str]) -> Result<(), Error> {
        run_mpv_command(self, command, args)
    }

    pub fn playlist_add(&self,
                        file: &str,
                        file_type: PlaylistAddTypeOptions,
                        option: PlaylistAddOptions)
                        -> Result<(), Error> {
        match file_type {
            PlaylistAddTypeOptions::File => {
                match option {
                    PlaylistAddOptions::Replace => {
                        run_mpv_command(self, "loadfile", &[file, "replace"])
                    }
                    PlaylistAddOptions::Append => {
                        run_mpv_command(self, "loadfile", &[file, "append"])
                    }
                    PlaylistAddOptions::AppendPlay => {
                        run_mpv_command(self, "loadfile", &[file, "append-play"])
                    }
                }
            }

            PlaylistAddTypeOptions::Playlist => {
                match option {
                    PlaylistAddOptions::Replace => {
                        run_mpv_command(self, "loadlist", &[file, "replace"])
                    }
                    PlaylistAddOptions::Append |
                    PlaylistAddOptions::AppendPlay => {
                        run_mpv_command(self, "loadlist", &[file, "append"])
                    }
                }
            }
        }

    }

    pub fn playlist_clear(&self) -> Result<(), Error> {
        run_mpv_command(self, "playlist-clear", &[])
    }

    pub fn playlist_move_id(&self, from: usize, to: usize) -> Result<(), Error> {
        run_mpv_command(self,
                        "playlist-remove",
                        &[&from.to_string(), &to.to_string()])
    }

    pub fn playlist_play_id(&self, id: usize) -> Result<(), Error> {
        set_mpv_property(self, "playlist-pos", id)
    }

    pub fn playlist_play_next(&self, id: usize) -> Result<(), Error> {
        match get_mpv_property::<usize>(self, "playlist-pos") {
            Ok(current_id) => {
                run_mpv_command(self,
                                "playlist-move",
                                &[&id.to_string(), &(current_id + 1).to_string()])
            }
            Err(msg) => Err(msg),
        }
    }

    pub fn playlist_remove_id(&self, id: usize) -> Result<(), Error> {
        run_mpv_command(self, "playlist-remove", &[&id.to_string()])
    }

    pub fn playlist_shuffle(&self) -> Result<(), Error> {
        run_mpv_command(self, "playlist-shuffle", &[])
    }

    pub fn seek(&self, seconds: f64, option: SeekOptions) -> Result<(), Error> {
        match option {
            SeekOptions::Absolute => {
                run_mpv_command(self, "seek", &[&seconds.to_string(), "absolute"])
            }
            SeekOptions::AbsolutePercent => {
                run_mpv_command(self, "seek", &[&seconds.to_string(), "absolute-percent"])
            }
            SeekOptions::Relative => {
                run_mpv_command(self, "seek", &[&seconds.to_string(), "relative"])
            }
            SeekOptions::RelativePercent => {
                run_mpv_command(self, "seek", &[&seconds.to_string(), "relative-percent"])
            }
        }
    }

    pub fn set_loop_file(&self, option: Switch) -> Result<(), Error> {
        let mut enabled = false;
        match option {
            Switch::On => enabled = true,
            Switch::Off => {}
            Switch::Toggle => {
                match get_mpv_property_string(self, "loop-file") {
                    Ok(value) => {
                        match value.as_ref() {
                            "false" => {
                                enabled = true;
                            }
                            _ => {
                                enabled = false;
                            }
                        }
                    }
                    Err(msg) => return Err(msg),
                }
            }
        }
        set_mpv_property(self, "loop-file", enabled)
    }

    pub fn set_loop_playlist(&self, option: Switch) -> Result<(), Error> {
        let mut enabled = false;
        match option {
            Switch::On => enabled = true,
            Switch::Off => {}
            Switch::Toggle => {
                match get_mpv_property_string(self, "loop-playlist") {
                    Ok(value) => {
                        match value.as_ref() {
                            "false" => {
                                enabled = true;
                            }
                            _ => {
                                enabled = false;
                            }
                        }
                    }
                    Err(msg) => return Err(msg),
                }
            }
        }
        set_mpv_property(self, "loop-playlist", enabled)
    }

    pub fn set_mute(&self, option: Switch) -> Result<(), Error> {
        let mut enabled = false;
        match option {
            Switch::On => enabled = true,
            Switch::Off => {}
            Switch::Toggle => {
                match get_mpv_property::<bool>(self, "mute") {
                    Ok(value) => {
                        enabled = !value;
                    }
                    Err(msg) => return Err(msg),
                }
            }
        }
        set_mpv_property(self, "mute", enabled)
    }

    /// #Description
    ///
    /// Sets the mpv property _<property>_ to _<value>_.
    ///
    /// ##Supported types
    /// - String
    /// - bool
    /// - f64
    /// - usize
    ///
    /// ##Input arguments
    ///
    /// - **property** defines the mpv property that should be retrieved
    /// - **value** defines the value of the given mpv property _<property>_
    ///
    /// #Example
    /// ```
    /// let mpv = Mpv::connect("/tmp/mpvsocket").unwrap();
    /// mpv.set_property("pause", true);
    /// ```
    pub fn set_property<T: SetPropertyTypeHandler<T>>(&self,
                                                      property: &str,
                                                      value: T)
                                                      -> Result<(), Error> {
        T::set_property_generic(self, property, value)
    }

    pub fn set_speed(&self, input_speed: f64, option: NumberChangeOptions) -> Result<(), Error> {
        match get_mpv_property::<f64>(self, "speed") {
            Ok(speed) => {
                match option {
                    NumberChangeOptions::Increase => {
                        set_mpv_property(self, "speed", speed + input_speed)
                    }

                    NumberChangeOptions::Decrease => {
                        set_mpv_property(self, "speed", speed - input_speed)
                    }

                    NumberChangeOptions::Absolute => set_mpv_property(self, "speed", input_speed),
                }
            }
            Err(msg) => Err(msg),
        }
    }

    pub fn set_volume(&self, input_volume: f64, option: NumberChangeOptions) -> Result<(), Error> {
        match get_mpv_property::<f64>(self, "volume") {
            Ok(volume) => {
                match option {
                    NumberChangeOptions::Increase => {
                        set_mpv_property(self, "volume", volume + input_volume)
                    }

                    NumberChangeOptions::Decrease => {
                        set_mpv_property(self, "volume", volume - input_volume)
                    }

                    NumberChangeOptions::Absolute => set_mpv_property(self, "volume", input_volume),
                }
            }
            Err(msg) => Err(msg),
        }
    }

    pub fn stop(&self) -> Result<(), Error> {
        run_mpv_command(self, "stop", &[])
    }

    pub fn toggle(&self) -> Result<(), Error> {
        match get_mpv_property::<bool>(self, "pause") {
            Ok(paused) => set_mpv_property(self, "pause", !paused),
            Err(msg) => Err(msg),
        }
    }
}