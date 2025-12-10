use clap::{Parser, Subcommand};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Mute or Unmute a channel (0=Unmute, 1=Mute)
    Mute {
        fader_index: u8,
        state: u8,
    },
    /// Set the input source for a channel
    Source {
        fader_index: u8,
        source_id: u32,
    },
    /// Set microphone type (Advanced)
    MicType {
        fader_index: u8,
        val: i64,
    },
    /// Set fader level (Virtual Only)
    Level {
        fader_index: u8,
        val: u32,
    },
    /// Simulate a screen touch
    Touch,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let socket_path = "/tmp/socket_bridge_control";

    let mut stream = UnixStream::connect(socket_path).await?;

    let cmd_str = match args.command {
        Commands::Mute { fader_index, state } => format!("mute {} {}", fader_index, state),
        Commands::Source { fader_index, source_id } => format!("source {} {}", fader_index, source_id),
        Commands::MicType { fader_index, val } => format!("mic_type {} {}", fader_index, val),
        Commands::Level { fader_index, val } => format!("level {} {}", fader_index, val),
        Commands::Touch => "touch".to_string(),
    };

    stream.write_all(cmd_str.as_bytes()).await?;
    println!("Sent: {}", cmd_str);

    Ok(())
}
