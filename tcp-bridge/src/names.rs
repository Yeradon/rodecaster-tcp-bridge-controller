//! Human-readable names for Rodecaster entities.
//! 
//! ## Supported Aliases
//! - **MixOutput**: `hp1-4`, `spk`, `rec`, `bt`, `cm1-3`
//! - **Source**: `mic1-4`, `combo12/23/34`, `bt`, `pad`, `cm1-3`
//! - **Fader**: `p1-6`, `v1-3`, `fader1-6`

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Mix output bus (where audio goes TO)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MixOutput {
    Headphone1, Headphone2, Headphone3, Headphone4,
    Speaker, Recording, Bluetooth,
    Usb1, Chat, Usb2,
    CallMe1, CallMe2, CallMe3,
}

impl MixOutput {
    pub fn to_index(&self) -> u8 {
        match self {
            Self::Headphone1 => 10, Self::Headphone2 => 11,
            Self::Headphone3 => 12, Self::Headphone4 => 13,
            Self::Speaker => 14, Self::Recording => 15, Self::Bluetooth => 16,
            Self::Usb1 => 17, Self::Chat => 18, Self::Usb2 => 19,
            Self::CallMe1 => 20, Self::CallMe2 => 21, Self::CallMe3 => 22,
        }
    }
}

impl FromStr for MixOutput {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "headphone1" | "hp1" => Ok(Self::Headphone1),
            "headphone2" | "hp2" => Ok(Self::Headphone2),
            "headphone3" | "hp3" => Ok(Self::Headphone3),
            "headphone4" | "hp4" => Ok(Self::Headphone4),
            "speaker" | "spk" => Ok(Self::Speaker),
            "recording" | "rec" => Ok(Self::Recording),
            "bluetooth" | "bt" => Ok(Self::Bluetooth),
            "usb1" => Ok(Self::Usb1),
            "chat" => Ok(Self::Chat),
            "usb2" => Ok(Self::Usb2),
            "callme1" | "cm1" => Ok(Self::CallMe1),
            "callme2" | "cm2" => Ok(Self::CallMe2),
            "callme3" | "cm3" => Ok(Self::CallMe3),
            _ => Err(format!("Unknown mix: {} (try: hp1, speaker, bt, cm1)", s)),
        }
    }
}

impl fmt::Display for MixOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Headphone1 => "headphone1", Self::Headphone2 => "headphone2",
            Self::Headphone3 => "headphone3", Self::Headphone4 => "headphone4",
            Self::Speaker => "speaker", Self::Recording => "recording",
            Self::Bluetooth => "bluetooth", Self::Usb1 => "usb1",
            Self::Chat => "chat", Self::Usb2 => "usb2",
            Self::CallMe1 => "callme1", Self::CallMe2 => "callme2", Self::CallMe3 => "callme3",
        })
    }
}

/// Audio source (where audio comes FROM)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Combo1, Combo2, Combo3, Combo4,
    Combo1_2, Combo2_3, Combo3_4,
    Usb1, Chat, Usb2, Bluetooth, SoundPad,
    VirtualGame, VirtualMusic, VirtualA, VirtualB,
    CallMe1, CallMe2, CallMe3,
}

impl Source {
    pub fn to_index(&self) -> u8 {
        match self {
            Self::Combo1 => 4, Self::Combo2 => 5, Self::Combo3 => 6, Self::Combo4 => 7,
            Self::Combo1_2 => 8, Self::Combo2_3 => 9, Self::Combo3_4 => 10,
            Self::Usb1 => 11, Self::Chat => 12, Self::Usb2 => 13,
            Self::Bluetooth => 14, Self::SoundPad => 15,
            Self::VirtualGame => 16, Self::VirtualMusic => 17, Self::VirtualA => 18, Self::VirtualB => 19,
            Self::CallMe1 => 1, Self::CallMe2 => 2, Self::CallMe3 => 3,
        }
    }

    /// CallMe sources require special protocol encoding
    pub fn is_callme(&self) -> bool {
        matches!(self, Self::CallMe1 | Self::CallMe2 | Self::CallMe3)
    }
}

impl FromStr for Source {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "combo1" | "mic1" => Ok(Self::Combo1),
            "combo2" | "mic2" => Ok(Self::Combo2),
            "combo3" | "mic3" => Ok(Self::Combo3),
            "combo4" | "mic4" => Ok(Self::Combo4),
            "combo1_2" | "combo12" => Ok(Self::Combo1_2),
            "combo2_3" | "combo23" => Ok(Self::Combo2_3),
            "combo3_4" | "combo34" => Ok(Self::Combo3_4),
            "usb1" => Ok(Self::Usb1),
            "chat" => Ok(Self::Chat),
            "usb2" => Ok(Self::Usb2),
            "bluetooth" | "bt" => Ok(Self::Bluetooth),
            "soundpad" | "pad" => Ok(Self::SoundPad),
            "virtualgame" | "game" | "vgame" => Ok(Self::VirtualGame),
            "virtualmusic" | "music" | "vmusic" => Ok(Self::VirtualMusic),
            "virtuala" | "va" => Ok(Self::VirtualA),
            "virtualb" | "vb" => Ok(Self::VirtualB),
            "callme1" | "cm1" => Ok(Self::CallMe1),
            "callme2" | "cm2" => Ok(Self::CallMe2),
            "callme3" | "cm3" => Ok(Self::CallMe3),
            _ => Err(format!("Unknown source: {} (try: combo1, bt, game, cm1)", s)),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Combo1 => "combo1", Self::Combo2 => "combo2",
            Self::Combo3 => "combo3", Self::Combo4 => "combo4",
            Self::Combo1_2 => "combo1_2", Self::Combo2_3 => "combo2_3", Self::Combo3_4 => "combo3_4",
            Self::Usb1 => "usb1", Self::Chat => "chat", Self::Usb2 => "usb2",
            Self::Bluetooth => "bluetooth", Self::SoundPad => "soundpad",
            Self::VirtualGame => "game", Self::VirtualMusic => "music",
            Self::VirtualA => "virtuala", Self::VirtualB => "virtualb",
            Self::CallMe1 => "callme1", Self::CallMe2 => "callme2", Self::CallMe3 => "callme3",
        })
    }
}

/// Physical or virtual fader
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fader {
    Physical1, Physical2, Physical3, Physical4, Physical5, Physical6,
    Virtual1, Virtual2, Virtual3,
}

impl Fader {
    pub fn to_index(&self) -> u8 {
        match self {
            Self::Physical1 => 0, Self::Physical2 => 1, Self::Physical3 => 2,
            Self::Physical4 => 3, Self::Physical5 => 4, Self::Physical6 => 5,
            Self::Virtual1 => 6, Self::Virtual2 => 7, Self::Virtual3 => 8,
        }
    }
}

impl FromStr for Fader {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "physical1" | "p1" | "fader1" | "1" => Ok(Self::Physical1),
            "physical2" | "p2" | "fader2" | "2" => Ok(Self::Physical2),
            "physical3" | "p3" | "fader3" | "3" => Ok(Self::Physical3),
            "physical4" | "p4" | "fader4" | "4" => Ok(Self::Physical4),
            "physical5" | "p5" | "fader5" | "5" => Ok(Self::Physical5),
            "physical6" | "p6" | "fader6" | "6" => Ok(Self::Physical6),
            "virtual1" | "v1" | "vfader1" => Ok(Self::Virtual1),
            "virtual2" | "v2" | "vfader2" => Ok(Self::Virtual2),
            "virtual3" | "v3" | "vfader3" => Ok(Self::Virtual3),
            _ => Err(format!("Unknown fader: {} (try: p1, fader1, v1)", s)),
        }
    }
}

impl fmt::Display for Fader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Physical1 => "physical1", Self::Physical2 => "physical2",
            Self::Physical3 => "physical3", Self::Physical4 => "physical4",
            Self::Physical5 => "physical5", Self::Physical6 => "physical6",
            Self::Virtual1 => "virtual1", Self::Virtual2 => "virtual2", Self::Virtual3 => "virtual3",
        })
    }
}
