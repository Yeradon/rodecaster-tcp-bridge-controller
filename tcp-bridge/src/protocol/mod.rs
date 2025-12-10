use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

pub mod mute;
pub mod source;
pub mod level;
pub mod touch;
pub mod mix;

pub use mute::*;
pub use source::*;
pub use level::*;
pub use touch::*;
pub use mix::*;

#[derive(Debug, Clone)]
pub struct Packet {
    pub header: u32,
    pub length: u32,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(payload: Vec<u8>) -> Self {
        Packet {
            header: 0xF2B49E2C, // Magic header
            length: payload.len() as u32,
            payload,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::new();
        wtr.write_u32::<LittleEndian>(self.header).unwrap();
        wtr.write_u32::<LittleEndian>(self.length).unwrap();
        wtr.write_all(&self.payload).unwrap();
        wtr
    }
}

pub trait RodeCommand {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8>;
}

pub fn extract_session_id(data: &[u8]) -> Option<Vec<u8>> {
    // Preamble: 2c 9e b4 f2
    if data.len() >= 12 
       && data[0] == 0x2c && data[1] == 0x9e && data[2] == 0xb4 && data[3] == 0xf2 
    {
        // Check for "ping" (70 69 6e 67) at offset 8 (no session id)
        if !(data[8] == 0x70 && data[9] == 0x69 && data[10] == 0x6e && data[11] == 0x67) {
             return Some(data[8..12].to_vec());
        }
    }
    None
}
