//! Unix socket listener for receiving commands from API server and CLI.
//! Uses JSON for type-safe command parsing.

use tokio::net::UnixListener;
use tokio::io::AsyncReadExt;
use tokio::sync::broadcast;

use crate::commands::{Command, MixAction};
use crate::names::Source;
use crate::protocol::{self, RodeCommand, MixCommand};

/// Internal command representation for the proxy
#[derive(Clone, Debug)]
pub enum ProxyCommand {
    Mute { fader_index: u8, mute: bool },
    Source { fader_index: u8, source_id: u32 },
    MicType { fader_index: u8, mic_type: u32 },
    Level { fader_index: u8, level: u32 },
    Touch,
    Mix { action: MixAction, mix_index: u8, source: Source },
}

pub async fn start_listener(tx: broadcast::Sender<ProxyCommand>) {
    let sock_path = "/tmp/socket_bridge_control";
    let _ = std::fs::remove_file(sock_path);
    
    let listener = UnixListener::bind(sock_path).expect("Failed to bind control socket");
    println!("[Listener] Listening on {} (JSON mode)", sock_path);
    
    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut buf = String::new();
                    if stream.read_to_string(&mut buf).await.is_ok() {
                        let commands = parse_commands(&buf);
                        for cmd in commands {
                            println!("[Listener] Received: {:?}", cmd);
                            let _ = tx.send(cmd);
                        }
                    }
                });
            }
            Err(e) => eprintln!("[Listener] Accept error: {}", e),
        }
    }
}

/// Parse input and return zero or more commands
fn parse_commands(input: &str) -> Vec<ProxyCommand> {
    // Try JSON first
    if let Ok(cmd) = serde_json::from_str::<Command>(input) {
        return convert_command(cmd);
    }
    
    // Fallback to legacy string format
    parse_legacy_command(input).into_iter().collect()
}

/// Convert a Command to one or more ProxyCommands
fn convert_command(cmd: Command) -> Vec<ProxyCommand> {
    match cmd {
        Command::Mix { action, mix, source } => {
            vec![ProxyCommand::Mix { action, mix_index: mix.to_index(), source }]
        }
        Command::Fader { fader, muted, source, level } => {
            let mut cmds = Vec::new();
            let idx = fader.to_index();
            
            // Add all specified fields as separate commands
            if let Some(m) = muted {
                cmds.push(ProxyCommand::Mute { fader_index: idx, mute: m });
            }
            if let Some(s) = source {
                cmds.push(ProxyCommand::Source { fader_index: idx, source_id: s.to_index() as u32 });
            }
            if let Some(l) = level {
                let level_val = (l.clamp(0.0, 1.0) * 65535.0) as u32;
                cmds.push(ProxyCommand::Level { fader_index: idx, level: level_val });
            }
            cmds
        }
        Command::Touch => vec![ProxyCommand::Touch],
    }
}

fn parse_legacy_command(input: &str) -> Option<ProxyCommand> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() { return None; }
    
    match parts[0] {
        "mute" if parts.len() >= 3 => {
            let fader = parts[1].parse().ok()?;
            let state: u8 = parts[2].parse().ok()?;
            Some(ProxyCommand::Mute { fader_index: fader, mute: state != 0 })
        }
        "source" if parts.len() >= 3 => {
            let fader = parts[1].parse().ok()?;
            let source = parts[2].parse().ok()?;
            Some(ProxyCommand::Source { fader_index: fader, source_id: source })
        }
        "level" if parts.len() >= 3 => {
            let fader = parts[1].parse().ok()?;
            let level = parts[2].parse().ok()?;
            Some(ProxyCommand::Level { fader_index: fader, level })
        }
        "touch" => Some(ProxyCommand::Touch),
        "mix_link" if parts.len() >= 3 => {
            let mix_index = parts[1].parse().ok()?;
            let source_index: u8 = parts[2].parse().ok()?;
            Some(ProxyCommand::Mix { 
                action: MixAction::Link, 
                mix_index, 
                source: index_to_source(source_index)? 
            })
        }
        "mix_unlink" if parts.len() >= 3 => {
            let mix_index = parts[1].parse().ok()?;
            let source_index: u8 = parts[2].parse().ok()?;
            Some(ProxyCommand::Mix { 
                action: MixAction::Unlink, 
                mix_index, 
                source: index_to_source(source_index)? 
            })
        }
        "callme_link" if parts.len() >= 3 => {
            let mix_index = parts[1].parse().ok()?;
            let callme_index: u8 = parts[2].parse().ok()?;
            let source = match callme_index {
                1 => Source::CallMe1, 2 => Source::CallMe2, 3 => Source::CallMe3,
                _ => return None,
            };
            Some(ProxyCommand::Mix { action: MixAction::Link, mix_index, source })
        }
        "callme_unlink" if parts.len() >= 3 => {
            let mix_index = parts[1].parse().ok()?;
            let callme_index: u8 = parts[2].parse().ok()?;
            let source = match callme_index {
                1 => Source::CallMe1, 2 => Source::CallMe2, 3 => Source::CallMe3,
                _ => return None,
            };
            Some(ProxyCommand::Mix { action: MixAction::Unlink, mix_index, source })
        }
        _ => None,
    }
}

fn index_to_source(idx: u8) -> Option<Source> {
    Some(match idx {
        4 => Source::Combo1, 5 => Source::Combo2, 6 => Source::Combo3, 7 => Source::Combo4,
        8 => Source::Combo1_2, 9 => Source::Combo2_3, 10 => Source::Combo3_4,
        11 => Source::Usb1, 12 => Source::Chat, 13 => Source::Usb2,
        14 => Source::Bluetooth, 15 => Source::SoundPad,
        16 => Source::VirtualGame, 17 => Source::VirtualMusic, 18 => Source::VirtualA, 19 => Source::VirtualB,
        1 => Source::CallMe1, 2 => Source::CallMe2, 3 => Source::CallMe3,
        _ => return None,
    })
}

impl ProxyCommand {
    /// Build all payloads for this command (some commands need multiple packets)
    pub fn build_payloads(&self, session_id: &[u8]) -> Vec<Vec<u8>> {
        match self {
            ProxyCommand::Mix { action, mix_index, source } => {
                MixCommand::new(*action, *mix_index, *source).build_payloads(session_id)
            }
            _ => vec![self.build_payload(session_id)],
        }
    }
}

impl RodeCommand for ProxyCommand {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        match self {
            ProxyCommand::Mute { fader_index, mute } => {
                protocol::ChannelOutputMute { fader_index: *fader_index, mute: *mute }
                    .build_payload(session_id)
            }
            ProxyCommand::Source { fader_index, source_id } => {
                protocol::ChannelInputSource { fader_index: *fader_index, source_id: *source_id }
                    .build_payload(session_id)
            }
            ProxyCommand::MicType { fader_index, mic_type } => {
                protocol::InputMicrophoneType { fader_index: *fader_index, mic_type: *mic_type }
                    .build_payload(session_id)
            }
            ProxyCommand::Level { fader_index, level } => {
                protocol::FaderLevel { fader_index: *fader_index, level: *level }
                    .build_payload(session_id)
            }
            ProxyCommand::Touch => {
                protocol::ScreenTouched.build_payload(session_id)
            }
            ProxyCommand::Mix { action, mix_index, source } => {
                MixCommand::new(*action, *mix_index, *source).build_payload(session_id)
            }
        }
    }
}
