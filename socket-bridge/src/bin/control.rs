use clap::Parser;
use std::io::Write;
use std::os::unix::net::UnixStream;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Command to send (e.g. "mute 1 1")
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.command.is_empty() {
        eprintln!("Usage: bridge-ctl <command> [args...]");
        return Ok(());
    }

    let cmd_str = args.command.join(" ");
    let socket_path = "/tmp/socket_bridge_control";

    let mut stream = UnixStream::connect(socket_path).map_err(|e| {
        eprintln!("Failed to connect to socket {}: {}", socket_path, e);
        e
    })?;

    stream.write_all(cmd_str.as_bytes())?;
    println!("Sent: '{}'", cmd_str);

    Ok(())
}
