use super::RodeCommand;
use byteorder::{LittleEndian, WriteBytesExt};

pub struct FaderLevel {
    pub fader_index: u8,
    pub level: u32,
}

impl RodeCommand for FaderLevel {
    fn build_payload(&self, _session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        // FaderLevel specific preamble from capture
        payload.extend_from_slice(&[0x01, 0x01, 0x02, 0x00]); 
        
        payload.push(0x01);

        // Base 0x04
        payload.push(0x04 + self.fader_index);
        payload.extend_from_slice(b"faderLevel\0");
        payload.push(0x01);
        payload.push(0x05);
        payload.push(0x01);
        payload.write_u32::<LittleEndian>(self.level).unwrap();
        
        payload
    }
}
