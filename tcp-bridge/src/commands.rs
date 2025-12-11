//! Unified command types for IPC between binaries.
//! Uses JSON serialization for type-safe communication.

use serde::{Deserialize, Serialize};
use crate::names::{MixOutput, Source, Fader};

/// Actions for mix commands
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MixAction {
    Link,    // Ensures routing is active (sends enable + link)
    Unlink,  // Sets to unlinked/fixed level
    Disable, // Mutes the routing
}

/// Unified command enum - serialized as JSON for IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Mix routing command (link/unlink/disable sources)
    Mix {
        action: MixAction,
        mix: MixOutput,
        source: Source,
    },
    /// Fader control command
    Fader {
        fader: Fader,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        muted: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<Source>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        level: Option<f32>,
    },
    /// Screen touch event
    Touch,
}

impl Command {
    /// Create a mix link command
    pub fn mix_link(mix: MixOutput, source: Source) -> Self {
        Command::Mix { action: MixAction::Link, mix, source }
    }

    /// Create a mix unlink command
    pub fn mix_unlink(mix: MixOutput, source: Source) -> Self {
        Command::Mix { action: MixAction::Unlink, mix, source }
    }

    /// Create a mute command
    pub fn mute(fader: Fader, muted: bool) -> Self {
        Command::Fader { fader, muted: Some(muted), source: None, level: None }
    }

    /// Create a level command
    pub fn level(fader: Fader, level: f32) -> Self {
        Command::Fader { fader, muted: None, source: None, level: Some(level) }
    }
}
