use minimp3::{Decoder, Error};

pub struct Song(pub Vec<f32>);

impl Song {
    pub fn decode_mp3<R: std::io::Read>(reader: R) -> Song {
        let mut decoder = Decoder::new(reader);
        let mut data = Vec::with_capacity(5_000_000);

        loop {
            match decoder.next_frame() {
                Ok(f) => {
                    assert_eq!(f.sample_rate, 48000, "Mp3 sample rate is not 48000");
                    let converted = f.data.into_iter().map(convert_sample_i16_f32);
                    data.extend(converted);
                }
                Err(Error::Eof) => break,
                Err(e) => panic!("{}", e),
            }
        }

        println!("{}", data.len());

        Song(data)
    }
}

fn convert_sample_i16_f32(n: i16) -> f32 {
    let f = n as f32;
    let max = i16::MAX as f32;

    2.0 * f / max + 1.0
}
