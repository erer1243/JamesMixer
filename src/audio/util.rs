/// This module contains utility stuff that isn't really pertinent to the main logic of the program.
use jack::{Client, Control, ProcessScope};

/// Process handler for jack. Could use jack::ClosureProcessHandler, but that would require
/// putting FnMut trait bounds and lifetimes on every surrounding struct.
pub struct JackBoxProcHandler(pub Box<dyn Send + FnMut(&Client, &ProcessScope) -> Control>);

impl jack::ProcessHandler for JackBoxProcHandler {
    fn process(&mut self, client: &Client, ps: &ProcessScope) -> Control {
        self.0(client, ps)
    }
}

/// Notification handler for jack. Same as one provided by examples except for two modifications:
/// It counts xruns and prints the count in the notification message, and it panics when jack shuts
/// down.
#[derive(Default)]
pub struct JackNotifs {
    xrun_count: usize,
}

impl jack::NotificationHandler for JackNotifs {
    // fn thread_init(&self, _: &jack::Client) {
    //     println!("JACK: thread init");
    // }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );

        panic!("Jack shutdown!");
    }

    // fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
    //     println!(
    //         "JACK: freewheel mode is {}",
    //         if is_enabled { "on" } else { "off" }
    //     );
    // }

    // fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
    //     println!("JACK: buffer size changed to {}", sz);
    //     jack::Control::Continue
    // }

    // fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
    //     println!("JACK: sample rate changed to {}", srate);
    //     jack::Control::Continue
    // }

    // fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
    //     println!(
    //         "JACK: {} client with name \"{}\"",
    //         if is_reg { "registered" } else { "unregistered" },
    //         name
    //     );
    // }

    // fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
    //     println!(
    //         "JACK: {} port with id {}",
    //         if is_reg { "registered" } else { "unregistered" },
    //         port_id
    //     );
    // }

    // fn port_rename(
    //     &mut self,
    //     _: &jack::Client,
    //     port_id: jack::PortId,
    //     old_name: &str,
    //     new_name: &str,
    // ) -> jack::Control {
    //     println!(
    //         "JACK: port with id {} renamed from {} to {}",
    //         port_id, old_name, new_name
    //     );
    //     jack::Control::Continue
    // }

    // fn ports_connected(
    //     &mut self,
    //     _: &jack::Client,
    //     port_id_a: jack::PortId,
    //     port_id_b: jack::PortId,
    //     are_connected: bool,
    // ) {
    //     println!(
    //         "JACK: ports with id {} and {} are {}",
    //         port_id_a,
    //         port_id_b,
    //         if are_connected {
    //             "connected"
    //         } else {
    //             "disconnected"
    //         }
    //     );
    // }

    // fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
    //     println!("JACK: graph reordered");
    //     jack::Control::Continue
    // }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        self.xrun_count += 1;
        println!("JACK: xrun occurred ({})", self.xrun_count);
        jack::Control::Continue
    }

    // fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
    //     println!(
    //         "JACK: {} latency has changed",
    //         match mode {
    //             jack::LatencyType::Capture => "capture",
    //             jack::LatencyType::Playback => "playback",
    //         }
    //     );
    // }
}
