pub fn print_hexdump(label: &str, data: &[u8]) {
    println!("{}: ({} bytes)", label, data.len());
    let len = data.len();
    let display_len = if len > 128 { 128 } else { len };
    
    for (i, chunk) in data[..display_len].chunks(16).enumerate() {
        print!("{:08x}  ", i * 16);
        for b in chunk {
            print!("{:02x} ", b);
        }
        // Padding for last line
        if chunk.len() < 16 {
            for _ in 0..(16 - chunk.len()) {
                print!("   ");
            }
        }
        print!(" |");
        for b in chunk {
            let c = *b as char;
            if c.is_ascii_graphic() || c == ' ' {
                print!("{}", c);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
    if len > 128 {
        println!("... ({} bytes truncated)", len - 128);
    }
    println!();
}

pub struct SnifferState {
    pub last_packet: Option<(String, Vec<u8>)>,
    pub repeat_count: usize,
}

impl SnifferState {
    pub fn new() -> Self {
        SnifferState {
            last_packet: None,
            repeat_count: 0,
        }
    }

    pub fn handle_packet(&mut self, direction: &str, data: &[u8]) {
        let current = (direction.to_string(), data.to_vec());
        if let Some((last_dir, last_data)) = &self.last_packet {
            if *last_dir == current.0 && *last_data == current.1 {
                self.repeat_count += 1;
            } else {
                if self.repeat_count > 0 {
                    println!("(Previous packet repeated {} times)", self.repeat_count);
                }
                print_hexdump(direction, data);
                self.last_packet = Some(current);
                self.repeat_count = 0;
            }
        } else {
             print_hexdump(direction, data);
             self.last_packet = Some(current);
             self.repeat_count = 0;
        }
    }
}
