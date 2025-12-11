//! CLI for controlling the Rodecaster via the proxy.
//! Uses human-readable names and unified commands.

use clap::{Parser, Subcommand, ValueEnum};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use serde_json;

use tcp_bridge::commands::{Command, MixAction};
use tcp_bridge::names::{MixOutput, Source, Fader};

#[derive(Parser, Debug)]
#[command(author, version, about = "Rodecaster CLI control")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Debug)]
enum CliMixAction {
    Link,
    Unlink,
    Disable,
}

impl From<CliMixAction> for MixAction {
    fn from(a: CliMixAction) -> Self {
        match a {
            CliMixAction::Link => MixAction::Link,
            CliMixAction::Unlink => MixAction::Unlink,
            CliMixAction::Disable => MixAction::Disable,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Mix routing: link/unlink/disable/enable sources
    /// Example: mix link hp1 bluetooth
    Mix {
        /// Action: link, unlink, disable, enable
        action: CliMixAction,
        /// Mix output (hp1-4, speaker, recording, bt, usb1, usb2, chat, cm1-3)
        mix: String,
        /// Source (combo1-4, combo12/23/34, usb1/2, bt, pad, game, music, va, vb, cm1-3)
        source: String,
    },
    /// Mute/unmute a fader
    Mute {
        /// Fader (p1-6, v1-3)
        fader: String,
        /// 0=unmute, 1=mute
        state: u8,
    },
    /// Set fader level (0-65535)
    Level {
        /// Fader (p1-6, v1-3)
        fader: String,
        /// Level value
        level: u32,
    },
    /// Simulate screen touch
    Touch,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let socket_path = "/tmp/socket_bridge_control";

    let cmd = match args.command {
        Commands::Mix { action, mix, source } => {
            let mix_output: MixOutput = mix.parse()
                .map_err(|e| format!("Invalid mix: {}", e))?;
            let src: Source = source.parse()
                .map_err(|e| format!("Invalid source: {}", e))?;
            Command::Mix { action: action.into(), mix: mix_output, source: src }
        }
        Commands::Mute { fader, state } => {
            let f: Fader = fader.parse()
                .map_err(|e| format!("Invalid fader: {}", e))?;
            Command::Fader { fader: f, muted: Some(state != 0), source: None, level: None }
        }
        Commands::Level { fader, level } => {
            let f: Fader = fader.parse()
                .map_err(|e| format!("Invalid fader: {}", e))?;
            Command::Fader { fader: f, muted: None, source: None, level: Some(level as f32 / 65535.0) }
        }
        Commands::Touch => Command::Touch,
    };

    let json = serde_json::to_string(&cmd)?;
    
    let mut stream = UnixStream::connect(socket_path).await?;
    stream.write_all(json.as_bytes()).await?;
    
    println!("Sent: {}", json);
    Ok(())
}
