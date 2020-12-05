use imgui::{ImStr, ImString};
use jack::{AsyncClient, AudioIn, AudioOut, Client, ClientOptions, Control, ProcessScope};
// use libpulse_binding::{
//     sample::{Spec, SAMPLE_S16NE},
//     stream::Direction,
// };
// use libpulse_simple_binding::Simple;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
// use std::sync::mpsc::{channel, Receiver, Sender};
// use std::thread;

// const PULSE_SPEC: &Spec = &Spec {
//     format: SAMPLE_S16NE,
//     rate: 48000,
//     channels: 2,
// };

// https://stackoverflow.com/questions/40929867/how-do-you-abstract-generics-in-nested-rust-types
// pub trait AudioFn {}
// impl<T: 'static + Send + FnMut(&Client, &ProcessScope) -> Control> AudioFn for T {}

pub struct Audio {
    // Available songs and their decoded data
    songs: BTreeMap<ImString, Song>,
    // Output connection
    // output: Simple,
    // mic_input: Simple,
    // line_input: Simple,
    // audio_thread: thread::JoinHandle<()>,

    // send: Sender<ThreadMessage>,
    jack_client: AsyncClient<Notifications, BoxedProcessHandler>,
}

struct Song(Vec<u8>);

impl Audio {
    pub fn init() -> Audio {
        // let (send, recv) = channel();
        // let audio_thread = thread::spawn(move || audio_thread_main(recv));

        let mut client_options = ClientOptions::empty();
        client_options.set(ClientOptions::NO_START_SERVER, true);
        let (client, _status) = Client::new("James' Mixer", client_options).unwrap();

        let mic_in = client
            .register_port("Microphone", AudioIn::default())
            .unwrap();

        let mut out_l = client.register_port("Output L", AudioOut::default()).unwrap();
        let mut out_r = client.register_port("Output R", AudioOut::default()).unwrap();

        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            out_l
                .as_mut_slice(ps)
                .clone_from_slice(mic_in.as_slice(ps));
            out_r
                .as_mut_slice(ps)
                .clone_from_slice(mic_in.as_slice(ps));
            Control::Continue
        };

        let process = BoxedProcessHandler(Box::new(process_callback));

        panic!();

        let jack_client = client.activate_async(Notifications, process).unwrap();

        Audio {
            songs: load_songs(),
            jack_client,
            // audio_thread,
            // send,
            // output: pulse_connect(Direction::Playback, "Output", None),
            // mic_input: pulse_connect(
            //     Direction::Record,
            //     "Mic Input",
            //     // Some("alsa_input.usb-Blue_Microphones_Yeti_Stereo_Microphone-00.analog-stereo"),
            //     None,
            // ),
            // line_input: pulse_connect(Direction::Record, "Line Input", None),
        }
    }

    pub fn song_list(&self) -> Vec<&ImStr> {
        self.songs.keys().map(|s| s.as_ref()).collect()
    }
}

impl Song {
    fn decode_mp3<R: std::io::Read>(data: R) -> Song {
        use minimp3::{Decoder, Error};

        let mut decoder = Decoder::new(data);
        let mut data = Vec::with_capacity(10_000_000);

        loop {
            match decoder.next_frame() {
                Ok(f) => {
                    let bytes = unsafe {
                        std::slice::from_raw_parts(f.data.as_ptr() as *const u8, f.data.len() * 2)
                    };
                    data.extend_from_slice(bytes);
                }
                Err(Error::Eof) => break,
                Err(e) => panic!("{}", e),
            }
        }

        Song(data)
    }
}

struct BoxedProcessHandler(Box<dyn Send + FnMut(&Client, &ProcessScope) -> Control>);

impl jack::ProcessHandler for BoxedProcessHandler {
    fn process(&mut self, client: &Client, ps: &ProcessScope) -> Control {
        self.0(client, ps)
    }
}

// enum ThreadMessage {
//     Quit,
// }

// fn audio_thread_main(recv: Receiver<ThreadMessage>) {
//     let output = pulse_connect(Direction::Playback, "Output", None);
//     let mic = pulse_connect(Direction::Record, "Input", None);

//     let mut buf = [0u8; 100];

//     println!("{:?} {:?}", output.get_latency(), mic.get_latency());

//     loop {
//         match recv.try_recv() {
//             Ok(ThreadMessage::Quit) => return,
//             _ => (),
//         }

//         mic.read(&mut buf).map_err(|e| e.to_string()).unwrap();
//         output.write(&buf).map_err(|e| e.to_string()).unwrap();
//     }
// }

// fn pulse_connect(dir: Direction, desc: &str, dev: Option<&str>) -> Simple {
//     Simple::new(
//         // Server (None = default)
//         None,
//         // Appliation name
//         "James' Mixer",
//         // Stream direction
//         dir,
//         // Device (None = default)
//         dev,
//         // Stream description
//         desc,
//         // Format spec
//         PULSE_SPEC,
//         // Channel map (None = default)
//         None,
//         // Buffering attributes (None = default)
//         None,
//     )
//     .unwrap_or_else(|e| {
//         panic!(
//             "\nPulse error\n  stream: {}\n  device: {:?}\n  error: {:?}\n",
//             desc,
//             dev,
//             e.to_string()
//         )
//     })
// }

fn load_songs() -> BTreeMap<ImString, Song> {
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
    let songs: BTreeMap<ImString, Song> = files
        .into_par_iter()
        .map(|(name, path)| (name, fs::read(path).unwrap()))
        .map(|(name, data)| (name, Song::decode_mp3(data.as_slice())))
        .collect();

    songs
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}
