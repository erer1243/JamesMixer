use jack::{Client, Control, ProcessScope};

/// Process handler for jack. Could use jack::ClosureProcessHandler, but that would require
/// putting FnMut trait bounds and lifetimes on every surrounding struct.
pub struct JackBoxProcHandler(pub Box<dyn Send + FnMut(&Client, &ProcessScope) -> Control>);

impl jack::ProcessHandler for JackBoxProcHandler {
    fn process(&mut self, client: &Client, ps: &ProcessScope) -> Control {
        self.0(client, ps)
    }
}

/// Notification handler for jack. Jack library has examples on how to create this.
#[derive(Default)]
pub struct JackNotifs {
    xrun_count: usize,
}

impl jack::NotificationHandler for JackNotifs {
    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );

        panic!("Jack shutdown!");
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        self.xrun_count += 1;
        println!("JACK: xrun occurred ({})", self.xrun_count);
        jack::Control::Continue
    }
}
