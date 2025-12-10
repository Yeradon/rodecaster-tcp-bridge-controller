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
    /// Link a source in a mix output
    /// Formula: prefix = source_index*13 + mix_index
    /// Known: Bluetooth=14, HP1=10, HP2=11
    MixLink {
        /// Mix index (HP1=10, HP2=11)
        mix_index: u8,
        /// Source index (Bluetooth=14, USB1=11, etc.)
        source_index: u8,
    },
    /// Unlink a source from a mix output
    MixUnlink {
        /// Mix index (HP1=10, HP2=11)
        mix_index: u8,
        /// Source index
        source_index: u8,
    },
    /// Disable a source in a mix (mutes audio)
    MixDisable {
        /// Mix index
        mix_index: u8,
        /// Source index
        source_index: u8,
        /// State: 2=active, 3=disabled
        state: u8,
    },
    /// Link a CallMe source (special 2-byte prefix encoding)
    CallMeLink {
        /// Mix index (HP1=10, HP2=11, etc.)
        mix_index: u8,
        /// CallMe index (1, 2, or 3)
        callme_index: u8,
    },
    /// Unlink a CallMe source
    CallMeUnlink {
        /// Mix index (HP1=10, HP2=11, etc.)
        mix_index: u8,
        /// CallMe index (1, 2, or 3)
        callme_index: u8,
    },
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
        Commands::MixLink { mix_index, source_index } => format!("mix_link {} {}", mix_index, source_index),
        Commands::MixUnlink { mix_index, source_index } => format!("mix_unlink {} {}", mix_index, source_index),
        Commands::MixDisable { mix_index, source_index, state } => format!("mix_disable {} {} {}", mix_index, source_index, state),
        Commands::CallMeLink { mix_index, callme_index } => format!("callme_link {} {}", mix_index, callme_index),
        Commands::CallMeUnlink { mix_index, callme_index } => format!("callme_unlink {} {}", mix_index, callme_index),
    };

    stream.write_all(cmd_str.as_bytes()).await?;
    println!("Sent: {}", cmd_str);

    Ok(())
}
