use libc::pid_t;
use std::collections::HashMap;

pub fn print_hexdump(prefix: &str, data: &[u8]) {
    println!("{} ({} bytes):", prefix, data.len());
    let width = 16;
    for (i, chunk) in data.chunks(width).enumerate() {
        print!("{:08x}  ", i * width);
        for (j, b) in chunk.iter().enumerate() {
            print!("{:02x} ", b);
            if j == 7 { print!(" "); }
        }
        if chunk.len() < width {
            let missing = width - chunk.len();
            let spaces = missing * 3 + (if chunk.len() <= 8 { 1 } else { 0 });
            for _ in 0..spaces { print!(" "); }
        }
        print!(" |");
        for b in chunk {
            if *b >= 32 && *b <= 126 { print!("{}", *b as char); }
            else { print!("."); }
        }
        println!("|");
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
