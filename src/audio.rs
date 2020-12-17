pub mod song;
mod util;

use song::Song;
use util::{JackBoxProcHandler, JackNotifs};

use imgui::{ImStr, ImString};
use jack::{AsyncClient, AudioIn, AudioOut, Client, ClientOptions, Control, ProcessScope};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering::Relaxed};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

/// Audio system. Connection to jack and state related to playing music.
pub struct Audio {
    /// Available songs and index in the songs vec (owned by Jack thread)
    song_index_map: BTreeMap<ImString, usize>,

    /// Jack context
    _jack_client: AsyncClient<JackNotifs, JackBoxProcHandler>,

    /// Channel with audio thread
    ac_send: Sender<AudioControl>,

    /// Atomics related to current state of music being played
    music: SharedAtomics,
}

impl Audio {
    /// Initialize and return audio system.
    pub fn init() -> Audio {
        // Setup jack
        let jack = Client::new("JamesMixer", ClientOptions::NO_START_SERVER)
            .expect("Jackd is not running")
            .0;

        // Load songs
        let (song_index_map, songs) = load_songs();

        // Create jack ports
        let mic_in = jack.register_port("mic_in", AudioIn::default()).unwrap();
        let line_in = jack.register_port("line_in", AudioIn::default()).unwrap();
        let mut output = jack.register_port("output", AudioOut::default()).unwrap();

        // Init MusicThread struct for closure
        let (ac_send, ac_recv) = channel();
        let shared = SharedAtomics {
            timestamp: Arc::new(AtomicUsize::new(0)),
            max_timestamp: Arc::new(AtomicUsize::new(0)),
            paused: Arc::new(AtomicBool::new(true)),
            mic_volume: Arc::new(AtomicU32::new(0)),
            line_volume: Arc::new(AtomicU32::new(0)),
            song_volume: Arc::new(AtomicU32::new(0)),
        };

        let mut music = MusicThread {
            ac_recv,
            songs,
            shared: shared.clone(),
            song: None,
        };

        // Callback closure that will be called by jack to update sound data buffer
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            audio_callback(
                output.as_mut_slice(ps),
                mic_in.as_slice(ps),
                line_in.as_slice(ps),
                &mut music,
            );
            Control::Continue
        };
        let process = JackBoxProcHandler(Box::new(process_callback));

        // Attach callbacks to jack
        let async_client = jack.activate_async(JackNotifs::default(), process).unwrap();

        // Connect jack ports. This requires a second client, due I suppose to a limitation in
        // the jack library.
        let jtemp = Client::new("JamesMixerTemp", ClientOptions::NO_START_SERVER)
            .unwrap()
            .0;

        // Macro to beautify jack port connection
        macro_rules! ports { ($($p1:expr => $p2:expr)*) => {
            $(match jtemp.connect_ports_by_name($p1, $p2) {
                Ok(()) => println!("{:18} => {:18} connected", $p1, $p2),
                Err(_) => println!("{:18} => {:18} NOT CONNECTED", $p1, $p2),
            };)*
        }; }

        // My setup for the show. In the future this could be read from a file, or deferred
        // to an external script or the user.
        ports! {
            "system:capture_1" => "JamesMixer:mic_in"
            "system:capture_2" => "JamesMixer:mic_in"
            "line_in:capture_1" => "JamesMixer:line_in"
            "line_in:capture_2" => "JamesMixer:line_in"
            "JamesMixer:output" => "system:playback_1"
            "JamesMixer:output" => "system:playback_2"
            "JamesMixer:output" => "darkice:mono"
        }

        Audio {
            song_index_map,
            _jack_client: async_client,
            ac_send,
            music: shared,
        }
    }

    /// Returns an alphabetized list of songs for rendering.
    pub fn song_list(&self) -> Vec<&ImStr> {
        self.song_index_map.keys().map(|s| s.as_ref()).collect()
    }

    /// Returns true if music is paused. This may not necessarily follow what was set using
    /// play_music and pause_music.
    pub fn get_paused(&self) -> bool {
        self.music.paused.load(Relaxed)
    }

    /// Returns the current timestamp and maximum timestamp of the current song in format (mm, ss).
    /// Returns 00:00 for both values if no song is loaded yet.
    pub fn music_timestamp(&self) -> ((usize, usize), (usize, usize)) {
        (
            samples_to_minsec(self.music.timestamp.load(Relaxed)),
            samples_to_minsec(self.music.max_timestamp.load(Relaxed)),
        )
    }

    /// Returns the current timestamp and maximum timestamp in samples. Returns 0 for both values
    /// if no song is loaded yet.
    pub fn music_samples(&self) -> (usize, usize) {
        (
            self.music.timestamp.load(Relaxed),
            self.music.max_timestamp.load(Relaxed),
        )
    }

    /// Pauses or attempts to unpause music. "Attempts" because if no song has been loaded yet,
    /// the paused state will not change, as it doesnt make sense to play an unselected song.
    pub fn set_paused(&self, value: bool) {
        self.ac_send.send(AudioControl::Paused(value)).unwrap();
    }

    /// Readies playing this song. Also pauses music and jumps to timestamp 00:00, and updates
    /// max_timestamp.
    pub fn load_song(&self, name: &ImStr) {
        let i = self.song_index_map[name];
        self.ac_send.send(AudioControl::Load(i)).unwrap();
    }

    /// Takes in a timestamp in minutes and seconds to jump to in the song, and pauses music.
    /// If the timestamp is out of range, or no song was loaded yet, it does nothing.
    pub fn jump_song(&self, minutes: usize, seconds: usize) {
        let samples = 48000 * (minutes * 60 + seconds);
        self.ac_send.send(AudioControl::JumpTo(samples)).unwrap();
    }

    /// Sets microphone volume
    pub fn set_mic_volume(&self, value: f32) {
        let v = value / 100.;
        self.music.mic_volume.store(v.to_bits(), Relaxed);
    }

    /// Sets line in volume
    pub fn set_line_volume(&self, value: f32) {
        let v = value / 100.;
        self.music.line_volume.store(v.to_bits(), Relaxed);
    }

    /// Sets song volume
    pub fn set_song_volume(&self, value: f32) {
        let v = value / 100.;
        self.music.song_volume.store(v.to_bits(), Relaxed);
    }
}

/// Struct that contains the atomics that are shared between ui and audio thread
#[derive(Clone)]
struct SharedAtomics {
    // Timestamp in samples.
    timestamp: Arc<AtomicUsize>,

    // Maximum possible timestamp in samples.
    max_timestamp: Arc<AtomicUsize>,

    // True if music is currently paused
    paused: Arc<AtomicBool>,

    // The volume of the three inputs
    mic_volume: Arc<AtomicU32>,
    line_volume: Arc<AtomicU32>,
    song_volume: Arc<AtomicU32>,
}

enum AudioControl {
    Paused(bool),
    Load(usize),
    JumpTo(usize),
}

struct MusicThread {
    // Receiver for controls from the ui thread
    ac_recv: Receiver<AudioControl>,

    // Song data that can be played given an index which is calculated in the ui thread
    songs: Vec<Song>,

    // Atomics shared between audio and ui threads
    shared: SharedAtomics,

    // Currently playing song
    song: Option<usize>,
}

fn audio_callback(out: &mut [f32], mic: &[f32], line: &[f32], music: &mut MusicThread) {
    use AudioControl::*;

    // A macro for easier working with atomics in the MusicThread struct
    #[rustfmt::skip]
    macro_rules! atom {
        ($name:ident)             => { music.shared.$name.load(Relaxed)        };
        ($name:ident = $val:expr) => { music.shared.$name.store($val, Relaxed) };
    }

    // Handle all music controls
    for c in music.ac_recv.try_iter() {
        match c {
            // Select song, and reset song playing variables
            Load(i) => {
                music.song = Some(i);
                atom!(timestamp = 0);
                atom!(max_timestamp = music.songs[i].data.len());
                atom!(paused = true);
            }

            // Set paused status. Pausing always works, but unpausing only works
            // when a song has been loaded, and music.song holds a value.
            Paused(p) => atom!(paused = p || music.song.is_none()),

            // If requested time within range of song, timestamp = time
            JumpTo(n) => {
                if n <= atom!(max_timestamp) {
                    atom!(timestamp = n);
                }
            }
        }
    }

    // Grab volume levels
    let mic_volume = f32::from_bits(atom!(mic_volume));
    let line_volume = f32::from_bits(atom!(line_volume));
    let song_volume = f32::from_bits(atom!(song_volume));

    if atom!(paused) {
        // Music paused - mix in only microphone and line-in.
        for i in 0..out.len() {
            let m = mic_volume * mic[i];
            let l = line_volume * line[i];
            out[i] = (m + l) / 3.0;
        }
    } else {
        // Music is not paused - mix in all three sources
        // Grab timestamp values
        let timestamp = atom!(timestamp);
        let max_timestamp = atom!(max_timestamp);

        // Step = number of song data samples that will be copied into the output buffer.
        // For most runs this will be equal to the length of the output, but if the music
        // data runs out mid-buffer, then step will be less.
        let step = out.len().min(max_timestamp - timestamp);

        // Get reference to current song data
        let i = music.song.unwrap();
        let song = &music.songs[i].data;

        // Copy song data mixed with mic and line
        for i in 0..step {
            let m = mic_volume * mic[i];
            let l = line_volume * line[i];
            let s = song_volume * song[timestamp + i];
            out[i] = (m + l + s) / 3.0;
        }

        // In the case that step < out.len(), fill the rest of out with a only a mix of mic and line
        for i in step..out.len() {
            let m = mic_volume * mic[i];
            let l = line_volume * line[i];
            out[i] = (m + l) / 3.0;
        }

        // Advance timestamp, and pause if hitting the end of the song
        let next_ts = timestamp + step;
        atom!(timestamp = next_ts);
        atom!(paused = next_ts == max_timestamp);
    }
}

/// Load and decode all songs from the ./music folder
fn load_songs() -> (BTreeMap<ImString, usize>, Vec<Song>) {
    // Vector of all file data
    let mut files: Vec<(ImString, PathBuf)> = Vec::new();

    // Load song files from disk
    for f in fs::read_dir("./music").unwrap() {
        // Grab file metadata
        let f = f.unwrap();
        let path = f.path();
        let name = f.file_name();

        // Turn file name into an ImString.
        let name_string = name.into_string().unwrap();
        assert!(name_string.is_ascii());
        let name_imstring = ImString::new(name_string);

        // Record name and path of this file
        files.push((name_imstring, path));
    }

    // Load and decode in parallel
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    let songs: Vec<(ImString, Song)> = files
        .into_par_iter()
        .inspect(|(name, _)| println!("Loading song {}", name))
        .map(|(name, path)| (name, fs::read(path).unwrap()))
        .map(|(name, data)| (name, Song::decode_mp3(data.as_slice())))
        .collect();

    // Create map from song name to index into song vec
    let index_map: BTreeMap<ImString, usize> = songs
        .iter()
        .enumerate()
        .map(|(i, (name, _))| (name.clone(), i))
        .collect();

    // Create song vec
    let songs: Vec<Song> = songs.into_iter().map(|(_, song)| song).collect();

    (index_map, songs)
}

/// Utility function to convert a number of samples into minutes and seconds
fn samples_to_minsec(samples: usize) -> (usize, usize) {
    let seconds = samples / 48000;
    (seconds / 60, seconds % 60)
}
