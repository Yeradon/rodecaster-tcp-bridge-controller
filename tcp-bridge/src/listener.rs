use tokio::net::UnixListener;
use tokio::io::AsyncReadExt;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum Command {
    ChannelOutputMute { fader_index: u8, mute: bool },
    ChannelInputSource { fader_index: u8, source_id: u32 },
    InputMicrophoneType { fader_index: u8, mic_type: u32 },
    FaderLevel { fader_index: u8, level: u32 },
    ScreenTouched,
    MixLink { mix_index: u8, source_index: u8 },
    MixUnlink { mix_index: u8, source_index: u8 },
    MixDisable { mix_index: u8, source_index: u8, state: u8 },
    CallMeLink { mix_index: u8, callme_index: u8 },
    CallMeUnlink { mix_index: u8, callme_index: u8 },
}

pub async fn start_listener(tx: broadcast::Sender<Command>) {
    let sock_path = "/tmp/socket_bridge_control";
    let _ = std::fs::remove_file(sock_path); // Cleanup
    
    let listener = UnixListener::bind(sock_path).expect("Failed to bind control socket");
    println!("[Listener] Listening on {}", sock_path);
    
    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut buf = String::new();
                    if let Ok(_) = stream.read_to_string(&mut buf).await {
                         if let Some(cmd) = parse_command(&buf) {
                            println!("[Listener] Received command: {:?}", buf);
                            let _ = tx.send(cmd);
                        }
                    }
                });
            }
            Err(e) => eprintln!("[Listener] Accept error: {}", e),
        }
    }
}

fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() { return None; }
    
    match parts[0] {
        "mute" => {
            if parts.len() < 3 { return None; }
            let fader = parts[1].parse::<u8>().ok()?;
            let state = parts[2].parse::<u8>().ok()?;
            Some(Command::ChannelOutputMute { fader_index: fader, mute: state != 0 })
        },
        "source" => {
            if parts.len() < 3 { return None; }
            let fader = parts[1].parse::<u8>().ok()?;
            let source = parts[2].parse::<u32>().ok()?;
            Some(Command::ChannelInputSource { fader_index: fader, source_id: source })
        },
        "mic_type" => {
            if parts.len() < 3 { return None; }
            let fader = parts[1].parse::<u8>().ok()?;
            let val = parts[2].parse::<i64>().ok()? as u32; // parse as i64 to handle -1, then cast
            Some(Command::InputMicrophoneType { fader_index: fader, mic_type: val })
        },
        "level" => {
            if parts.len() < 3 { return None; }
            let fader = parts[1].parse::<u8>().ok()?;
            let val = parts[2].parse::<u32>().ok()?;
            Some(Command::FaderLevel { fader_index: fader, level: val })
        },
        "touch" => Some(Command::ScreenTouched),
        "mix_link" => {
            if parts.len() < 3 { return None; }
            let mix_index = parts[1].parse::<u8>().ok()?;
            let source_index = parts[2].parse::<u8>().ok()?;
            Some(Command::MixLink { mix_index, source_index })
        },
        "mix_unlink" => {
            if parts.len() < 3 { return None; }
            let mix_index = parts[1].parse::<u8>().ok()?;
            let source_index = parts[2].parse::<u8>().ok()?;
            Some(Command::MixUnlink { mix_index, source_index })
        },
        "mix_disable" => {
            if parts.len() < 4 { return None; }
            let mix_index = parts[1].parse::<u8>().ok()?;
            let source_index = parts[2].parse::<u8>().ok()?;
            let state = parts[3].parse::<u8>().ok()?;
            Some(Command::MixDisable { mix_index, source_index, state })
        },
        "callme_link" => {
            if parts.len() < 3 { return None; }
            let mix_index = parts[1].parse::<u8>().ok()?;
            let callme_index = parts[2].parse::<u8>().ok()?;
            Some(Command::CallMeLink { mix_index, callme_index })
        },
        "callme_unlink" => {
            if parts.len() < 3 { return None; }
            let mix_index = parts[1].parse::<u8>().ok()?;
            let callme_index = parts[2].parse::<u8>().ok()?;
            Some(Command::CallMeUnlink { mix_index, callme_index })
        },
        _ => None,
    }
}

impl crate::protocol::RodeCommand for Command {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        match self {
            Command::ChannelOutputMute { fader_index, mute } => {
                let cmd = crate::protocol::ChannelOutputMute { fader_index: *fader_index, mute: *mute };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::ChannelInputSource { fader_index, source_id } => {
                let cmd = crate::protocol::ChannelInputSource { fader_index: *fader_index, source_id: *source_id };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::InputMicrophoneType { fader_index, mic_type } => {
                let cmd = crate::protocol::InputMicrophoneType { fader_index: *fader_index, mic_type: *mic_type };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::FaderLevel { fader_index, level } => {
                let cmd = crate::protocol::FaderLevel { fader_index: *fader_index, level: *level };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::ScreenTouched => {
                let cmd = crate::protocol::ScreenTouched;
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::MixLink { mix_index, source_index } => {
                let cmd = crate::protocol::MixLinkRequest { mix_index: *mix_index, source_index: *source_index };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::MixUnlink { mix_index, source_index } => {
                let cmd = crate::protocol::MixUnlinkRequest { mix_index: *mix_index, source_index: *source_index };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::MixDisable { mix_index, source_index, state } => {
                let cmd = crate::protocol::MixDisabled { mix_index: *mix_index, source_index: *source_index, state: *state };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::CallMeLink { mix_index, callme_index } => {
                let cmd = crate::protocol::CallMeLinkRequest { mix_index: *mix_index, callme_index: *callme_index };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
            Command::CallMeUnlink { mix_index, callme_index } => {
                let cmd = crate::protocol::CallMeUnlinkRequest { mix_index: *mix_index, callme_index: *callme_index };
                crate::protocol::RodeCommand::build_payload(&cmd, session_id)
            },
        }
    }
}
