use super::RodeCommand;

/// Calculate the mix prefix byte from source index and mix index.
/// 
/// Formula discovered from packet analysis:
/// `prefix = source_index * 13 + mix_index`
/// 
/// The payload value is fixed at 0x02 (not the mix_index).
pub fn calculate_mix_prefix(source_index: u8, mix_index: u8) -> u8 {
    source_index * 13 + mix_index
}

/// Known source indices (confirmed via testing)
#[allow(non_snake_case)]
pub mod SourceIndex {
    pub const COMBO_1: u8 = 4;      // Combo input 1
    pub const COMBO_2: u8 = 5;      // Combo input 2
    pub const COMBO_3: u8 = 6;      // Combo input 3
    pub const COMBO_4: u8 = 7;      // Combo input 4
    pub const COMBO_1_2: u8 = 8;      // Combo input 1+2
    pub const COMBO_2_3: u8 = 9;      // Combo input 2+3
    pub const COMBO_3_4: u8 = 10;      // Combo input 3+4
    pub const USB_1: u8 = 11;       // USB 1
    pub const CHAT: u8 = 12;        // Chat/CallMe
    pub const USB_2: u8 = 13;       // USB 2
    pub const BLUETOOTH: u8 = 14;   // Bluetooth
    pub const SOUNDPAD: u8 = 15;    // SoundPad
}

/// Known mix indices (output buses) - confirmed via testing
#[allow(non_snake_case)]
pub mod MixIndex {
    pub const HEADPHONE_1: u8 = 10;
    pub const HEADPHONE_2: u8 = 11;
    pub const HEADPHONE_3: u8 = 12;
    pub const HEADPHONE_4: u8 = 13;
    pub const SPEAKER: u8 = 14;
    pub const RECORDING: u8 = 15;
    pub const BLUETOOTH: u8 = 16;
    pub const USB_1: u8 = 17;
    pub const CHAT: u8 = 18;
    pub const USB_2: u8 = 19;
    pub const CALLME_1: u8 = 20;
    pub const CALLME_2: u8 = 21;
    pub const CALLME_3: u8 = 22;
}

/// Link a source in a specific mix output
pub struct MixLinkRequest {
    pub mix_index: u8,     // Mix output (HP1=10, HP2=11)
    pub source_index: u8,  // Source index (Bluetooth=14)
}

impl RodeCommand for MixLinkRequest {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        
        let prefix = calculate_mix_prefix(self.source_index, self.mix_index);
        payload.push(prefix);
        payload.extend_from_slice(b"mixLinkRequest\0");
        
        // Payload: 01 07 08 01 01 02 01 01 02 (fixed 02 values)
        payload.push(0x01);
        payload.push(0x07);
        payload.push(0x08);
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);  // Fixed command value
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);  // Fixed command value
        
        payload
    }
}

/// Unlink a source from a specific mix output
pub struct MixUnlinkRequest {
    pub mix_index: u8,     // Mix output (HP1=10, HP2=11)
    pub source_index: u8,  // Source index (Bluetooth=14)
}

impl RodeCommand for MixUnlinkRequest {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        
        let prefix = calculate_mix_prefix(self.source_index, self.mix_index);
        payload.push(prefix);
        payload.extend_from_slice(b"mixUnlinkRequest\0");
        
        // Payload: 01 07 08 01 01 02 01 01 02 (fixed 02 values)
        payload.push(0x01);
        payload.push(0x07);
        payload.push(0x08);
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);
        
        payload
    }
}

/// Disable/activate a source in a mix
pub struct MixDisabled {
    pub mix_index: u8,     // Mix output
    pub source_index: u8,  // Source index
    pub state: u8,         // 02 = active, 03 = disabled
}

impl RodeCommand for MixDisabled {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        
        let prefix = calculate_mix_prefix(self.source_index, self.mix_index);
        payload.push(prefix);
        payload.extend_from_slice(b"mixDisabled\0");
        
        payload.push(0x01);
        payload.push(0x01);
        payload.push(self.state);
        
        payload
    }
}

/// CallMe sources use a different encoding with 2-byte prefix
/// First byte: 4 + mix_index (HP1=14, HP2=15, etc.)
/// Second byte: callme_index (1, 2, or 3)
/// 
/// IMPORTANT: CallMe also uses a different session ID: 01 01 01 02
pub struct CallMeUnlinkRequest {
    pub mix_index: u8,     // Mix output (HP1=10, HP2=11, etc.)
    pub callme_index: u8,  // 1, 2, or 3
}

impl RodeCommand for CallMeUnlinkRequest {
    fn build_payload(&self, _session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        
        // CallMe uses special session ID: 01 01 01 02
        payload.extend_from_slice(&[0x01, 0x01, 0x01, 0x02]);
        
        // CallMe prefix: first = 4 + mix_index, second = callme_index
        payload.push(4 + self.mix_index);
        payload.push(self.callme_index);
        payload.extend_from_slice(b"mixUnlinkRequest\0");
        
        payload.push(0x01);
        payload.push(0x07);
        payload.push(0x08);
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);
        
        payload
    }
}

/// CallMe link request
pub struct CallMeLinkRequest {
    pub mix_index: u8,     // Mix output (HP1=10, HP2=11, etc.)
    pub callme_index: u8,  // 1, 2, or 3
}

impl RodeCommand for CallMeLinkRequest {
    fn build_payload(&self, _session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        
        // CallMe uses special session ID: 01 01 01 02
        payload.extend_from_slice(&[0x01, 0x01, 0x01, 0x02]);
        
        payload.push(4 + self.mix_index);
        payload.push(self.callme_index);
        payload.extend_from_slice(b"mixLinkRequest\0");
        
        payload.push(0x01);
        payload.push(0x07);
        payload.push(0x08);
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);
        
        payload
    }
}



