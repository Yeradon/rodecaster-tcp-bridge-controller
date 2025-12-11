//! Mix routing protocol commands.
//! 
//! Link sends both enable + link packets to work from any state.

use super::RodeCommand;
use crate::names::Source;
use crate::commands::MixAction;

/// Formula: prefix = source_index * 13 + mix_index
fn calculate_mix_prefix(source_index: u8, mix_index: u8) -> u8 {
    source_index * 13 + mix_index
}

/// Unified mix command - handles link/unlink/disable
pub struct MixCommand {
    pub action: MixAction,
    pub mix_index: u8,
    pub source: Source,
}

impl MixCommand {
    pub fn new(action: MixAction, mix_index: u8, source: Source) -> Self {
        Self { action, mix_index, source }
    }
    
    /// Build all payloads needed for this command
    /// Link returns two packets (enable first, then link)
    pub fn build_payloads(&self, session_id: &[u8]) -> Vec<Vec<u8>> {
        if self.source.is_callme() {
            vec![self.build_callme_payload()]
        } else {
            match self.action {
                MixAction::Link => {
                    // Send enable first (to handle Disabled state), then link
                    vec![
                        self.build_enable_payload(session_id),
                        self.build_link_payload(session_id),
                    ]
                }
                _ => vec![self.build_regular_payload(session_id)],
            }
        }
    }
    
    fn build_enable_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        payload.push(calculate_mix_prefix(self.source.to_index(), self.mix_index));
        payload.extend_from_slice(b"mixDisabled\0");
        payload.extend_from_slice(&[0x01, 0x01, 0x03]); // 03 = enabled
        payload
    }
    
    fn build_link_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        payload.push(calculate_mix_prefix(self.source.to_index(), self.mix_index));
        payload.extend_from_slice(b"mixLinkRequest\0");
        payload.extend_from_slice(&[0x01, 0x07, 0x08, 0x01, 0x01, 0x02, 0x01, 0x01, 0x02]);
        payload
    }
    
    fn build_regular_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        payload.push(calculate_mix_prefix(self.source.to_index(), self.mix_index));
        
        match self.action {
            MixAction::Link => unreachable!(), // Handled separately
            MixAction::Unlink => {
                payload.extend_from_slice(b"mixUnlinkRequest\0");
                payload.extend_from_slice(&[0x01, 0x07, 0x08, 0x01, 0x01, 0x02, 0x01, 0x01, 0x02]);
            }
            MixAction::Disable => {
                payload.extend_from_slice(b"mixDisabled\0");
                payload.extend_from_slice(&[0x01, 0x01, 0x02]); // 02 = disabled
            }
        }
        payload
    }

    fn build_callme_payload(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0x01, 0x01, 0x01, 0x02]); // Special session ID
        payload.push(4 + self.mix_index);
        payload.push(self.source.to_index());
        
        match self.action {
            MixAction::Link => {
                payload.extend_from_slice(b"mixLinkRequest\0");
                payload.extend_from_slice(&[0x01, 0x07, 0x08, 0x01, 0x01, 0x02, 0x01, 0x01, 0x02]);
            }
            MixAction::Unlink => {
                payload.extend_from_slice(b"mixUnlinkRequest\0");
                payload.extend_from_slice(&[0x01, 0x07, 0x08, 0x01, 0x01, 0x02, 0x01, 0x01, 0x02]);
            }
            MixAction::Disable => {} // Not supported for CallMe
        }
        payload
    }
}

// Keep RodeCommand impl for backward compat (uses first payload only)
impl RodeCommand for MixCommand {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        self.build_payloads(session_id).into_iter().next().unwrap_or_default()
    }
}
