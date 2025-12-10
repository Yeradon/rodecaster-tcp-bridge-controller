use super::RodeCommand;

pub struct ChannelOutputMute {
    pub fader_index: u8,
    pub mute: bool,
}

impl RodeCommand for ChannelOutputMute {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        
        // Base 0x1C
        payload.push(0x1C + self.fader_index);
        payload.extend_from_slice(b"channelOutputMute\0");
        payload.push(0x01); // Type
        payload.push(0x01); // Val
        payload.push(if self.mute { 0x02 } else { 0x03 });
        
        payload
    }
}
