use std::str::FromStr;
use std::fmt;

/// Human-readable mix output names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixOutput {
    Headphone1,
    Headphone2,
    Headphone3,
    Headphone4,
    Speaker,
    Recording,
    Bluetooth,
    Usb1,
    Chat,
    Usb2,
    CallMe1,
    CallMe2,
    CallMe3,
}

impl MixOutput {
    pub fn to_index(&self) -> u8 {
        match self {
            MixOutput::Headphone1 => 10,
            MixOutput::Headphone2 => 11,
            MixOutput::Headphone3 => 12,
            MixOutput::Headphone4 => 13,
            MixOutput::Speaker => 14,
            MixOutput::Recording => 15,
            MixOutput::Bluetooth => 16,
            MixOutput::Usb1 => 17,
            MixOutput::Chat => 18,
            MixOutput::Usb2 => 19,
            MixOutput::CallMe1 => 20,
            MixOutput::CallMe2 => 21,
            MixOutput::CallMe3 => 22,
        }
    }
}

impl FromStr for MixOutput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "headphone1" | "hp1" => Ok(MixOutput::Headphone1),
            "headphone2" | "hp2" => Ok(MixOutput::Headphone2),
            "headphone3" | "hp3" => Ok(MixOutput::Headphone3),
            "headphone4" | "hp4" => Ok(MixOutput::Headphone4),
            "speaker" | "spk" => Ok(MixOutput::Speaker),
            "recording" | "rec" => Ok(MixOutput::Recording),
            "bluetooth" | "bt" => Ok(MixOutput::Bluetooth),
            "usb1" => Ok(MixOutput::Usb1),
            "chat" => Ok(MixOutput::Chat),
            "usb2" => Ok(MixOutput::Usb2),
            "callme1" | "cm1" => Ok(MixOutput::CallMe1),
            "callme2" | "cm2" => Ok(MixOutput::CallMe2),
            "callme3" | "cm3" => Ok(MixOutput::CallMe3),
            _ => Err(format!("Unknown mix output: {}", s)),
        }
    }
}

impl fmt::Display for MixOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MixOutput::Headphone1 => write!(f, "headphone1"),
            MixOutput::Headphone2 => write!(f, "headphone2"),
            MixOutput::Headphone3 => write!(f, "headphone3"),
            MixOutput::Headphone4 => write!(f, "headphone4"),
            MixOutput::Speaker => write!(f, "speaker"),
            MixOutput::Recording => write!(f, "recording"),
            MixOutput::Bluetooth => write!(f, "bluetooth"),
            MixOutput::Usb1 => write!(f, "usb1"),
            MixOutput::Chat => write!(f, "chat"),
            MixOutput::Usb2 => write!(f, "usb2"),
            MixOutput::CallMe1 => write!(f, "callme1"),
            MixOutput::CallMe2 => write!(f, "callme2"),
            MixOutput::CallMe3 => write!(f, "callme3"),
        }
    }
}

/// Human-readable source names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Combo1,
    Combo2,
    Combo3,
    Combo4,
    Combo1_2,
    Combo2_3,
    Combo3_4,
    Usb1,
    Chat,
    Usb2,
    Bluetooth,
    SoundPad,
    CallMe1,
    CallMe2,
    CallMe3,
}

impl Source {
    /// Get the source index for regular mix commands
    pub fn to_index(&self) -> u8 {
        match self {
            Source::Combo1 => 4,
            Source::Combo2 => 5,
            Source::Combo3 => 6,
            Source::Combo4 => 7,
            Source::Combo1_2 => 8,
            Source::Combo2_3 => 9,
            Source::Combo3_4 => 10,
            Source::Usb1 => 11,
            Source::Chat => 12,
            Source::Usb2 => 13,
            Source::Bluetooth => 14,
            Source::SoundPad => 15,
            // CallMe sources return their index (1, 2, 3) for special handling
            Source::CallMe1 => 1,
            Source::CallMe2 => 2,
            Source::CallMe3 => 3,
        }
    }
    
    /// Returns true if this source requires special CallMe encoding
    pub fn is_callme(&self) -> bool {
        matches!(self, Source::CallMe1 | Source::CallMe2 | Source::CallMe3)
    }
}

impl FromStr for Source {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "combo1" | "mic1" => Ok(Source::Combo1),
            "combo2" | "mic2" => Ok(Source::Combo2),
            "combo3" | "mic3" => Ok(Source::Combo3),
            "combo4" | "mic4" => Ok(Source::Combo4),
            "combo1_2" | "combo12" => Ok(Source::Combo1_2),
            "combo2_3" | "combo23" => Ok(Source::Combo2_3),
            "combo3_4" | "combo34" => Ok(Source::Combo3_4),
            "usb1" => Ok(Source::Usb1),
            "chat" => Ok(Source::Chat),
            "usb2" => Ok(Source::Usb2),
            "bluetooth" | "bt" => Ok(Source::Bluetooth),
            "soundpad" | "pad" => Ok(Source::SoundPad),
            "callme1" | "cm1" => Ok(Source::CallMe1),
            "callme2" | "cm2" => Ok(Source::CallMe2),
            "callme3" | "cm3" => Ok(Source::CallMe3),
            _ => Err(format!("Unknown source: {}", s)),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Combo1 => write!(f, "combo1"),
            Source::Combo2 => write!(f, "combo2"),
            Source::Combo3 => write!(f, "combo3"),
            Source::Combo4 => write!(f, "combo4"),
            Source::Combo1_2 => write!(f, "combo1_2"),
            Source::Combo2_3 => write!(f, "combo2_3"),
            Source::Combo3_4 => write!(f, "combo3_4"),
            Source::Usb1 => write!(f, "usb1"),
            Source::Chat => write!(f, "chat"),
            Source::Usb2 => write!(f, "usb2"),
            Source::Bluetooth => write!(f, "bluetooth"),
            Source::SoundPad => write!(f, "soundpad"),
            Source::CallMe1 => write!(f, "callme1"),
            Source::CallMe2 => write!(f, "callme2"),
            Source::CallMe3 => write!(f, "callme3"),
        }
    }
}

/// Human-readable fader names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fader {
    Physical1,
    Physical2,
    Physical3,
    Physical4,
    Physical5,
    Physical6,
    Virtual1,
    Virtual2,
    Virtual3,
}

impl Fader {
    pub fn to_index(&self) -> u8 {
        match self {
            Fader::Physical1 => 0,
            Fader::Physical2 => 1,
            Fader::Physical3 => 2,
            Fader::Physical4 => 3,
            Fader::Physical5 => 4,
            Fader::Physical6 => 5,
            Fader::Virtual1 => 6,
            Fader::Virtual2 => 7,
            Fader::Virtual3 => 8,
        }
    }
}

impl FromStr for Fader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "physical1" | "p1" | "fader1" | "1" => Ok(Fader::Physical1),
            "physical2" | "p2" | "fader2" | "2" => Ok(Fader::Physical2),
            "physical3" | "p3" | "fader3" | "3" => Ok(Fader::Physical3),
            "physical4" | "p4" | "fader4" | "4" => Ok(Fader::Physical4),
            "physical5" | "p5" | "fader5" | "5" => Ok(Fader::Physical5),
            "physical6" | "p6" | "fader6" | "6" => Ok(Fader::Physical6),
            "virtual1" | "v1" | "vfader1" => Ok(Fader::Virtual1),
            "virtual2" | "v2" | "vfader2" => Ok(Fader::Virtual2),
            "virtual3" | "v3" | "vfader3" => Ok(Fader::Virtual3),
            _ => Err(format!("Unknown fader: {}", s)),
        }
    }
}

impl fmt::Display for Fader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fader::Physical1 => write!(f, "physical1"),
            Fader::Physical2 => write!(f, "physical2"),
            Fader::Physical3 => write!(f, "physical3"),
            Fader::Physical4 => write!(f, "physical4"),
            Fader::Physical5 => write!(f, "physical5"),
            Fader::Physical6 => write!(f, "physical6"),
            Fader::Virtual1 => write!(f, "virtual1"),
            Fader::Virtual2 => write!(f, "virtual2"),
            Fader::Virtual3 => write!(f, "virtual3"),
        }
    }
}

/// CallMe source (1, 2, or 3) - handled specially due to different encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallMeSource(pub u8);

impl CallMeSource {
    pub fn index(&self) -> u8 {
        self.0
    }
}

impl FromStr for CallMeSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "callme1" | "cm1" | "1" => Ok(CallMeSource(1)),
            "callme2" | "cm2" | "2" => Ok(CallMeSource(2)),
            "callme3" | "cm3" | "3" => Ok(CallMeSource(3)),
            _ => Err(format!("Unknown CallMe source: {} (expected 1, 2, or 3)", s)),
        }
    }
}
