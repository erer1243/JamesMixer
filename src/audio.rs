mod song;
mod util;

use song::Song;
use util::{JackBoxProcHandler, JackNotifs};

use imgui::{ImStr, ImString};
use jack::{AsyncClient, AudioIn, AudioOut, Client, ClientOptions, Control, ProcessScope};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Audio system. Connection to jack and state related to playing music.
pub struct Audio {
    // Available songs and index in the songs vec (owned by Jack thread)
    song_index_map: BTreeMap<ImString, usize>,

    // Jack context
    _jack_client: AsyncClient<JackNotifs, JackBoxProcHandler>,
}

impl Audio {
    pub fn init() -> Audio {
        // Connect to jack
        let jack = Client::new("JamesMixer", ClientOptions::NO_START_SERVER)
            .expect("Jack is not running")
            .0;

        // Load songs
        let (song_index_map, songs) = load_songs();

        // Create jack ports
        let mic_in = jack.register_port("mic_in", AudioIn::default()).unwrap();
        let line_in = jack.register_port("line_in", AudioIn::default()).unwrap();
        let mut output = jack.register_port("output", AudioOut::default()).unwrap();

        // Init closure
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let output = output.as_mut_slice(ps);
            let mic_in = mic_in.as_slice(ps);
            let line_in = line_in.as_slice(ps);

            Control::Continue
        };
        let process = JackBoxProcHandler(Box::new(process_callback));

        // Attach callbacks to jack
        let jack_client = jack.activate_async(JackNotifs::default(), process).unwrap();

        Audio {
            song_index_map,
            _jack_client: jack_client,
        }
    }

    pub fn song_list(&self) -> Vec<&ImStr> {
        self.song_index_map.keys().map(|s| s.as_ref()).collect()
    }
}

enum MusicControl {
    PauseSong,
    PlaySong,
}

struct Music {
    songs: Vec<Song>,
    current_song: Option<usize>,
}

/// Load all songs from the ./music folder
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
        .map(|(name, path)| (name, fs::read(path).unwrap()))
        .map(|(name, data)| (name, Song::decode_mp3(data.as_slice())))
        .collect();

    // Create map from song name to index into song vec
    let index_map: BTreeMap<ImString, usize> = songs
        .iter()
        .enumerate()
        .map(|(i, (name, song))| (name.clone(), i))
        .collect();

    // Create song vec
    let songs: Vec<Song> = songs.into_iter().map(|(_, song)| song).collect();

    (index_map, songs)
}
