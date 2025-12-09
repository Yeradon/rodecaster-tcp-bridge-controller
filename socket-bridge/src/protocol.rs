use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

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

pub fn build_channel_output_mute(fader_index: u8, mute: bool) -> Vec<u8> {
    // Structure based on captured logs:
    // Header (8 bytes) handling done by Packet::new
    // Payload:
    // 01 01 01 01 (Prefix)
    // [FaderID char] "channelOutputMute\0"
    // 01 [Val] [Action]
    
    // Fader IDs from docs/logs:
    // Fader 1: 0x22 ('"')
    // Fader 2: 0x23 ('#')
    // Fader 3: 0x24 ('$')
    // etc.
    // Let's assume fader_index 1-mapped to these chars for now.
    // Fader 1 (Index 0?) -> 0x22? 
    // Captures showed: Fader 1 -> 0x22. 
    // Let's take fader_id as u8 directly.
    
    // Fader mapping: 1 -> 0x22, 2 -> 0x23, etc.
    // Base offset: 0x21 (33)
    let fader_char = 0x21 + fader_index;
    payload.push(fader_char);
    
    // "channelOutputMute" + null
    payload.extend_from_slice(b"channelOutputMute\0");
    
    // Type (?)
    payload.push(0x01);
    
    // Val (? always 01?)
    payload.push(0x01);
    
    // Action: 0x02 = Mute, 0x03 = Unmute
    payload.push(if mute { 0x02 } else { 0x03 });
    
    payload
}
