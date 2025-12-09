use std::os::unix::net::UnixListener;
use std::io::Read;
use std::sync::mpsc::Sender;
use std::thread;

pub enum Command {
    ChannelOutputMute { fader_index: u8, mute: bool },
    // Add more commands here
}

pub fn start_listener(tx: Sender<Command>) {
    thread::spawn(move || {
        let sock_path = "/tmp/socket_bridge_control";
        let _ = std::fs::remove_file(sock_path); // Cleanup
        
        let listener = UnixListener::bind(sock_path).expect("Failed to bind control socket");
        println!("[Listener] Listening on {}", sock_path);
        
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buf = String::new();
                    if let Ok(_) = stream.read_to_string(&mut buf) {
                        if let Some(cmd) = parse_command(&buf) {
                            println!("[Listener] Received command: {:?}", buf);
                            let _ = tx.send(cmd);
                        }
                    }
                }
                Err(err) => eprintln!("[Listener] Connection failed: {}", err),
            }
        }
    });
}

fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() { return None; }
    
    match parts[0] {
        "mute" => {
            // mute <fader_index> <0/1>
            if parts.len() < 3 { return None; }
            let fader = parts[1].parse::<u8>().ok()?;
            let state = parts[2].parse::<u8>().ok()?;
            Some(Command::ChannelOutputMute { fader_index: fader, mute: state != 0 })
        },
        _ => None,
    }
}
