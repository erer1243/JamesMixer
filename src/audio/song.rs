use minimp3::{Decoder, Error};
use std::io::Read;

pub struct Song {
    pub data: Vec<f32>,
}

impl Song {
    pub fn decode_mp3<R: Read>(reader: R) -> Song {
        let mut decoder = Decoder::new(reader);
        let mut data = Vec::with_capacity(10_000_000);

        loop {
            match decoder.next_frame() {
                Ok(f) => {
                    assert_eq!(f.sample_rate, 48000, "Mp3 sample rate is not 48000");
                    let converted = f.data.into_iter().map(|n| n as f32 / i16::MAX as f32);
                    data.extend(converted);
                }
                Err(Error::Eof) => break,
                Err(e) => panic!("{}", e),
            }
        }

        Song { data }
    }
}
