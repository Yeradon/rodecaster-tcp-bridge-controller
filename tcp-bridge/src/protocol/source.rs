use super::RodeCommand;
use byteorder::{LittleEndian, WriteBytesExt};

pub struct ChannelInputSource {
    pub fader_index: u8,
    pub source_id: u32,
}

impl RodeCommand for ChannelInputSource {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        
        // Base 0x1C
        payload.push(0x1C + self.fader_index);
        payload.extend_from_slice(b"channelInputSource\0");
        payload.push(0x01);
        payload.push(0x05); // Type: Integer
        payload.push(0x01); // Count
        payload.write_u32::<LittleEndian>(self.source_id).unwrap();
        
        payload
    }
}

pub struct InputMicrophoneType {
    pub fader_index: u8,
    pub mic_type: u32,
}

impl RodeCommand for InputMicrophoneType {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        
        // Base 0x1C
        payload.push(0x1C + self.fader_index);
        payload.extend_from_slice(b"inputMicrophoneType\0");
        payload.push(0x01);
        payload.push(0x05);
        payload.push(0x01);
        payload.write_u32::<LittleEndian>(self.mic_type).unwrap();
        
        payload
    }
}
