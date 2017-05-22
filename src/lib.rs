extern crate serde;
extern crate serde_json;

pub mod ipc;

use std::collections::HashMap;
use ipc::*;

pub type Socket = String;

pub enum NumberChangeOptions {
    Absolute,
    Increase,
    Decrease,
}

pub enum SeekOptions {
    Relative,
    Absolute,
    RelativePercent,
    AbsolutePercent,
}

pub enum PlaylistAddOptions {
    Replace,
    Append,
    AppendPlay,
}

pub enum Switch {
    On,
    Off,
    Toggle,
}

pub struct Playlist {
    pub socket: Socket,
    pub entries: Vec<PlaylistEntry>,
}

pub trait GetPropertyTypeHandler: Sized {
    fn get_property_generic(socket: &str, property: &str) -> Result<Self, String>;
}

impl GetPropertyTypeHandler for bool {
    fn get_property_generic(socket: &str, property: &str) -> Result<bool, String> {
        get_mpv_property::<bool>(socket, property)
    }
}

impl GetPropertyTypeHandler for String {
    fn get_property_generic(socket: &str, property: &str) -> Result<String, String> {
        get_mpv_property::<String>(socket, property)
    }
}

impl GetPropertyTypeHandler for f64 {
    fn get_property_generic(socket: &str, property: &str) -> Result<f64, String> {
        get_mpv_property::<f64>(socket, property)
    }
}

impl GetPropertyTypeHandler for usize {
    fn get_property_generic(socket: &str, property: &str) -> Result<usize, String> {
        get_mpv_property::<usize>(socket, property)
    }
}

impl GetPropertyTypeHandler for Vec<PlaylistEntry> {
    fn get_property_generic(socket: &str, property: &str) -> Result<Vec<PlaylistEntry>, String> {
        get_mpv_property::<Vec<PlaylistEntry>>(socket, property)
    }
}

impl GetPropertyTypeHandler for HashMap<String, String> {
    fn get_property_generic(socket: &str,
                            property: &str)
                            -> Result<HashMap<String, String>, String> {
        get_mpv_property::<HashMap<String, String>>(socket, property)
    }
}

pub trait SetPropertyTypeHandler<T> {
    fn set_property_generic(socket: &str, property: &str, value: T) -> Result<(), String>;
}

impl SetPropertyTypeHandler<bool> for bool {
    fn set_property_generic(socket: &str, property: &str, value: bool) -> Result<(), String> {
        set_mpv_property::<bool>(socket, property, value)
    }
}

impl SetPropertyTypeHandler<String> for String {
    fn set_property_generic(socket: &str, property: &str, value: String) -> Result<(), String> {
        set_mpv_property::<String>(socket, property, value)
    }
}

impl SetPropertyTypeHandler<f64> for f64 {
    fn set_property_generic(socket: &str, property: &str, value: f64) -> Result<(), String> {
        set_mpv_property::<f64>(socket, property, value)
    }
}

impl SetPropertyTypeHandler<usize> for usize {
    fn set_property_generic(socket: &str, property: &str, value: usize) -> Result<(), String> {
        set_mpv_property::<usize>(socket, property, value)
    }
}

pub trait PlaylistHandler {
    fn get_from(socket: Socket) -> Result<Playlist, String>;
    fn shuffle(&mut self) -> &mut Playlist;
    fn remove_id(&mut self, id: usize) -> &mut Playlist;
    fn move_entry(&mut self, from: usize, to: usize) -> &mut Playlist;
    fn current_id(&self) -> Option<usize>;
}

impl PlaylistHandler for Playlist {
    fn get_from(socket: Socket) -> Result<Playlist, String> {
        match get_mpv_property(&socket, "playlist") {
            Ok(playlist) => {
                Ok(Playlist {
                       socket: socket,
                       entries: playlist,
                   })
            }
            Err(why) => Err(why),
        }
    }

    fn shuffle(&mut self) -> &mut Playlist {
        if let Err(error_msg) = run_mpv_command(&self.socket, "playlist-shuffle", &vec![]) {
            panic!("Error: {}", error_msg);
        }
        if let Ok(mut playlist_entries) =
            get_mpv_property::<Vec<PlaylistEntry>>(&self.socket, "playlist") {
            if self.entries.len() == playlist_entries.len() {
                for (i, entry) in playlist_entries.drain(0..).enumerate() {
                    self.entries[i] = entry;
                }
            }
        }
        self
    }

    fn remove_id(&mut self, id: usize) -> &mut Playlist {
        self.entries.remove(id);
        if let Err(error_msg) = run_mpv_command(&self.socket,
                                                "playlist-remove",
                                                &vec![&id.to_string()]) {
            panic!("Error: {}", error_msg);
        }
        self
    }

    fn move_entry(&mut self, from: usize, to: usize) -> &mut Playlist {
        if from != to {
            if let Err(error_msg) = run_mpv_command(&self.socket,
                                                    "playlist-move",
                                                    &vec![&from.to_string(), &to.to_string()]) {
                panic!("Error: {}", error_msg);
            }
            if from < to {
                self.entries[from].id = to - 1;
                self.entries[to].id = to - 2;
                for i in from..to - 2 {
                    self.entries[i + 1].id = i;
                }
                self.entries.sort_by_key(|entry| entry.id);
            } else if from > to {
                self.entries[from].id = to;
                for i in to..from - 1 {
                    self.entries[i].id = i + 1;
                }
                self.entries.sort_by_key(|entry| entry.id);
            }
        }
        self
    }

    fn current_id(&self) -> Option<usize> {
        for entry in self.entries.iter() {
            if entry.current {
                return Some(entry.id);
            }
        }
        None
    }
}

pub trait Commands {
    fn get_metadata(&self) -> Result<HashMap<String, String>, String>;
    fn get_playlist(&self) -> Result<Playlist, String>;
    fn get_property<T: GetPropertyTypeHandler>(&self, property: &str) -> Result<T, String>;
    fn get_property_string(&self, property: &str) -> Result<String, String>;
    fn kill(&self) -> Result<(), String>;
    fn next(&self) -> Result<(), String>;
    fn pause(&self) -> Result<(), String>;
    fn playlist_add(&self, file: &str, option: PlaylistAddOptions) -> Result<(), String>;
    fn playlist_clear(&self) -> Result<(), String>;
    fn playlist_move_id(&self, from: usize, to: usize) -> Result<(), String>;
    fn playlist_play_id(&self, id: usize) -> Result<(), String>;
    fn playlist_play_next(&self, id: usize) -> Result<(), String>;
    fn playlist_shuffle(&self) -> Result<(), String>;
    fn playlist_remove_id(&self, id: usize) -> Result<(), String>;
    fn prev(&self) -> Result<(), String>;
    fn restart(&self) -> Result<(), String>;
    fn run_command(&self, command: &str, args: &Vec<&str>) -> Result<(), String>;
    fn seek(&self, seconds: f64, option: SeekOptions) -> Result<(), String>;
    fn set_loop_file(&self, option: Switch) -> Result<(), String>;
    fn set_loop_playlist(&self, option: Switch) -> Result<(), String>;
    fn set_mute(&self, option: Switch) -> Result<(), String>;
    fn set_property<T: SetPropertyTypeHandler<T>>(&self,
                                                  property: &str,
                                                  value: T)
                                                  -> Result<(), String>;
    fn set_speed(&self, input_volume: f64, option: NumberChangeOptions) -> Result<(), String>;
    fn set_volume(&self, input_volume: f64, option: NumberChangeOptions) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn toggle(&self) -> Result<(), String>;
}

impl Commands for Socket {
    fn get_metadata(&self) -> Result<HashMap<String, String>, String> {
        match get_mpv_property(self, "metadata") {
            Ok(map) => Ok(map),
            Err(err) => Err(err),
        }
    }

    fn get_playlist(&self) -> Result<Playlist, String> {
        Playlist::get_from(self.to_string())
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
    ///
    /// ##Input arguments
    ///
    /// - **socket** defines the socket that ipc connects to
    /// - **property** defines the mpv property that should be retrieved
    ///
    /// #Example
    /// ```
    /// let mpv: Socket = String::from(matches.value_of("socket").unwrap());
    /// let paused: bool = mpv.get_property("pause").unwrap();
    /// let title: String = mpv.get_property("media-title").unwrap();
    /// ```
    fn get_property<T: GetPropertyTypeHandler>(&self, property: &str) -> Result<T, String> {
        T::get_property_generic(self, property)
    }

    /// #Description
    ///
    /// Retrieves the property value from mpv. Implemented for the following types:
    /// The result is always of type String, regardless of the type of the value of the mpv property
    ///
    /// ##Input arguments
    ///
    /// - **socket** defines the socket that ipc connects to
    /// - **property** defines the mpv property that should be retrieved
    ///
    /// #Example
    ///
    /// ```
    /// let mpv: Socket = String::from(matches.value_of("socket").unwrap());
    /// let title = mpv.get_property_string("media-title").unwrap();
    /// ```
    fn get_property_string(&self, property: &str) -> Result<String, String> {
        get_mpv_property_string(self, property)
    }

    fn kill(&self) -> Result<(), String> {
        run_mpv_command(self, "quit", &vec![])
    }

    fn next(&self) -> Result<(), String> {
        run_mpv_command(self, "playlist-next", &vec![])
    }

    fn pause(&self) -> Result<(), String> {
        set_mpv_property(self, "pause", true)
    }

    fn prev(&self) -> Result<(), String> {
        run_mpv_command(self, "playlist-prev", &vec![])
    }

    fn restart(&self) -> Result<(), String> {
        run_mpv_command(self, "seek", &vec!["0", "absolute"])
    }

    /// #Description
    ///
    /// Runs mpv commands. The arguments are passed as a String-Vector reference:
    ///
    /// #Example
    /// ```
    /// let mpv: Socket = String::from(matches.value_of("socket").unwrap());
    ///
    /// //Run command 'playlist-shuffle' which takes no arguments
    /// mpv.run_command("playlist-shuffle", &vec![]);
    ///
    /// //Run command 'seek' which in this case takes two arguments
    /// mpv.run_command("seek", &vec!["0", "absolute"]);
    /// ```
    fn run_command(&self, command: &str, args: &Vec<&str>) -> Result<(), String> {
        run_mpv_command(self, command, args)
    }

    fn playlist_add(&self, file: &str, option: PlaylistAddOptions) -> Result<(), String> {
        match option {
            PlaylistAddOptions::Replace => {
                run_mpv_command(self, "loadfile", &vec![file, "replace"])
            }
            PlaylistAddOptions::Append => run_mpv_command(self, "loadfile", &vec![file, "append"]),
            PlaylistAddOptions::AppendPlay => {
                run_mpv_command(self, "loadfile", &vec![file, "append-play"])
            }
        }
    }

    fn playlist_clear(&self) -> Result<(), String> {
        run_mpv_command(self, "playlist-clear", &vec![])
    }

    fn playlist_move_id(&self, from: usize, to: usize) -> Result<(), String> {
        run_mpv_command(self,
                        "playlist-remove",
                        &vec![&from.to_string(), &to.to_string()])
    }

    fn playlist_play_id(&self, id: usize) -> Result<(), String> {
        set_mpv_property(self, "playlist-pos", id)
    }

    fn playlist_play_next(&self, id: usize) -> Result<(), String> {
        match Playlist::get_from(self.to_string()) {
            Ok(playlist) => {
                if let Some(current_id) = playlist.current_id() {
                    run_mpv_command(self,
                                    "playlist-move",
                                    &vec![&id.to_string(), &(current_id + 1).to_string()])
                } else {
                    Err("There is no file playing at the moment.".to_string())
                }
            }
            Err(why) => Err(why),
        }
    }

    fn playlist_remove_id(&self, id: usize) -> Result<(), String> {
        run_mpv_command(self, "playlist-remove", &vec![&id.to_string()])
    }

    fn playlist_shuffle(&self) -> Result<(), String> {
        run_mpv_command(self, "playlist-shuffle", &vec![])
    }

    fn seek(&self, seconds: f64, option: SeekOptions) -> Result<(), String> {
        match option {
            SeekOptions::Absolute => {
                run_mpv_command(self, "seek", &vec![&seconds.to_string(), "absolute"])
            }
            SeekOptions::AbsolutePercent => {
                run_mpv_command(self,
                                "seek",
                                &vec![&seconds.to_string(), "absolute-percent"])
            }
            SeekOptions::Relative => {
                run_mpv_command(self, "seek", &vec![&seconds.to_string(), "relative"])
            }
            SeekOptions::RelativePercent => {
                run_mpv_command(self,
                                "seek",
                                &vec![&seconds.to_string(), "relative-percent"])
            }
        }
    }

    fn set_loop_file(&self, option: Switch) -> Result<(), String> {
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

    fn set_loop_playlist(&self, option: Switch) -> Result<(), String> {
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

    fn set_mute(&self, option: Switch) -> Result<(), String> {
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
    /// let mpv: Socket = String::from(matches.value_of("socket").unwrap());
    /// mpv.set_property("pause", true);
    /// ```
    fn set_property<T: SetPropertyTypeHandler<T>>(&self,
                                                  property: &str,
                                                  value: T)
                                                  -> Result<(), String> {
        T::set_property_generic(self, property, value)
    }

    fn set_speed(&self, input_speed: f64, option: NumberChangeOptions) -> Result<(), String> {
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

    fn set_volume(&self, input_volume: f64, option: NumberChangeOptions) -> Result<(), String> {
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

    fn stop(&self) -> Result<(), String> {
        run_mpv_command(self, "stop", &vec![])
    }

    fn toggle(&self) -> Result<(), String> {
        match get_mpv_property::<bool>(self, "pause") {
            Ok(paused) => set_mpv_property(self, "pause", !paused),
            Err(msg) => Err(msg),
        }
    }
}