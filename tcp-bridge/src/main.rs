mod listener;
mod sniffer;

// Re-export from library
pub use tcp_bridge::{names, protocol, commands};

use clap::Parser;
use std::net::SocketAddr;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use socket2::{Socket, Domain, Type};
use tokio::sync::broadcast;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = "127.0.0.2")]
    bind_ip: String,

    #[arg(long, default_value_t = 9000)]
    bind_port: u16,

    #[arg(long, default_value = "127.0.0.1")]
    target_ip: String,

    #[arg(long, default_value_t = 2345)]
    target_port: u16,

    #[arg(long, default_value = "127.0.0.2")]
    source_ip: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Command Channel
    let (cmd_tx, _cmd_rx) = broadcast::channel(16);
    
    // Start Listener
    {
        let tx = cmd_tx.clone();
        tokio::spawn(async move {
            listener::start_listener(tx).await;
        });
    }

    let bind_addr: SocketAddr = format!("{}:{}", args.bind_ip, args.bind_port).parse()?;
    let listener = TcpListener::bind(bind_addr).await?;
    println!("Proxy listening on {}", bind_addr);
    println!("Targeting {}:{} (Binding Source: {})", args.target_ip, args.target_port, args.source_ip);

    while let Ok((mut client_socket, addr)) = listener.accept().await {
        println!("New connection from: {}", addr);
        
        let target_ip = args.target_ip.clone();
        let target_port = args.target_port;
        let source_ip = args.source_ip.clone();
        
        // Each connection gets a receiver
        let mut cmd_rx = cmd_tx.subscribe();

        tokio::spawn(async move {
            match connect_to_target(&target_ip, target_port, &source_ip).await {
                Ok(mut server_socket) => {
                    let (mut client_reader, mut client_writer) = client_socket.split();
                    let (mut server_reader, mut server_writer) = server_socket.split();
                    
                    // Internal channel for S->C Injection (Loopback)
                    let (inject_tx, mut inject_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(16);

                    // Client -> Server (PLUS Injection)
                    let client_to_server = async {
                        let mut buf = [0u8; 4096];
                        let mut sniffer = sniffer::SnifferState::new();
                        let mut current_session_id = vec![0x01, 0x01, 0x01, 0x01]; // Default
                        
                        // We need to move inject_tx into this block
                        let inject_tx = inject_tx; 

                        loop {
                            tokio::select! {
                                res = client_reader.read(&mut buf) => {
                                    match res {
                                        Ok(0) => break, // EOF
                                        Ok(n) => {
                                            sniffer.handle_packet("C->S", &buf[..n]);
                                            
                                            // Dynamic Session ID Sniffing
                                            if let Some(sid) = protocol::extract_session_id(&buf[..n]) {
                                                 current_session_id = sid;
                                                 // println!("[Proxy] Sniffed SessionID: {:02x?}", current_session_id);
                                            }

                                            if let Err(e) = server_writer.write_all(&buf[..n]).await {
                                                eprintln!("Failed to write to server: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Client read error: {}", e);
                                            break;
                                        }
                                    }
                                }
                                Ok(cmd) = cmd_rx.recv() => {
                                    // Inject Command(s)!
                                    println!("[Proxy] Injecting Command: {:?}", cmd);
                                    let payloads = cmd.build_payloads(&current_session_id);
                                    
                                    for (i, payload) in payloads.iter().enumerate() {
                                        // Rate limit: delay between multi-packet commands
                                        if i > 0 {
                                            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                                        }
                                        
                                        // UI SYNC: Loopback injection
                                        let packet = protocol::Packet::new(payload.clone());
                                        let bytes = packet.to_bytes();
                                        if let Err(e) = inject_tx.send(bytes.clone()).await {
                                             eprintln!("Failed to queue UI Sync: {}", e);
                                        }
                                        
                                        sniffer::print_hexdump("INJECTED", &bytes);
                                        
                                        if let Err(e) = server_writer.write_all(&bytes).await {
                                            eprintln!("Failed to inject command: {}", e);
                                        }
                                    }
                                    println!("[Proxy] Injection Sent ({} packets).", payloads.len());
                                }
                            }
                        }
                        // Shutdown
                        let _ = server_writer.shutdown().await;
                    };

                    // Server -> Client
                    let server_to_client = async {
                        let mut buf = [0u8; 4096];
                        let mut sniffer = sniffer::SnifferState::new();
                        loop {
                            tokio::select! {
                                res = server_reader.read(&mut buf) => {
                                    match res {
                                        Ok(0) => break, // EOF
                                        Ok(n) => {
                                            sniffer.handle_packet("S->C", &buf[..n]);
                                            if let Err(e) = client_writer.write_all(&buf[..n]).await {
                                                eprintln!("Failed to write to client: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Server read error: {}", e);
                                            break;
                                        }
                                    }
                                }
                                Some(injected_bytes) = inject_rx.recv() => {
                                    // Handle Loopback Injection
                                    sniffer::print_hexdump("INJECTED_LOOPBACK", &injected_bytes);
                                    if let Err(e) = client_writer.write_all(&injected_bytes).await {
                                        eprintln!("Failed to write loopback to client: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        // Shutdown
                        let _ = client_writer.shutdown().await;
                    };

                    tokio::join!(client_to_server, server_to_client);
                }
                Err(e) => eprintln!("Failed to connect to target: {}", e),
            }
        });
    }

    Ok(())
}

async fn connect_to_target(ip: &str, port: u16, source_ip: &str) -> io::Result<TcpStream> {
    let target_addr: SocketAddr = format!("{}:{}", ip, port).parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let source_addr: SocketAddr = format!("{}:0", source_ip).parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    socket.bind(&source_addr.into())?;
    socket.connect(&target_addr.into())?;
    
    // Convert to Tokio Stream
    let std_stream: std::net::TcpStream = socket.into();
    std_stream.set_nonblocking(true)?;
    TcpStream::from_std(std_stream)
}

