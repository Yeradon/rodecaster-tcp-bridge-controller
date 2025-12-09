mod common;
mod injector;
mod listener;
mod protocol;
mod sniffer;

use clap::Parser;
use libc::{
    c_void, iovec, pid_t, ptrace, waitpid, PTRACE_ATTACH, PTRACE_DETACH, PTRACE_GETREGSET,
    PTRACE_O_TRACESYSGOOD, PTRACE_SETOPTIONS, PTRACE_SYSCALL, SIGTRAP, WIFEXITED, WIFSIGNALED,
    WIFSTOPPED, WSTOPSIG, NT_PRSTATUS
};
use std::mem;
use std::ptr;
use std::sync::mpsc;
use common::UserPtRegs;

// AArch64 constants
const SYS_READ: u64 = 63;
const SYS_WRITE: u64 = 64;
const SYS_READV: u64 = 65;
const SYS_WRITEV: u64 = 66;
const SYS_SENDTO: u64 = 206;
const SYS_RECVFROM: u64 = 207;
const SYS_SENDMSG: u64 = 211;
const SYS_RECVMSG: u64 = 212;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    pid: pid_t,
    fd: u64,
}

#[derive(Debug)]
struct SyscallContext {
    syscall_nr: u64,
    fd: u64,
    buf_addr: u64,
    count: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let pid = args.pid;
    let target_fd = args.fd;

    // Start Command Listener
    let (tx, rx) = mpsc::channel();
    listener::start_listener(tx);

    // Prepare Injector (ptrace helper)
    let injector = injector::Injector::new(pid, target_fd);

    println!("Attaching to PID: {}", pid);
    unsafe {
        let res = ptrace(PTRACE_ATTACH, pid, ptr::null_mut::<c_void>(), ptr::null_mut::<c_void>());
        if res == -1 {
            return Err(std::io::Error::last_os_error().into());
        }
        
        let mut status = 0;
        waitpid(pid, &mut status, 0);
        
        let res = ptrace(PTRACE_SETOPTIONS, pid, ptr::null_mut::<c_void>(), PTRACE_O_TRACESYSGOOD as *mut c_void);
        if res == -1 {
            eprintln!("PTRACE_SETOPTIONS failed: {}", std::io::Error::last_os_error());
        }
    }
    println!("Attached. Sniffing FD: {}", target_fd);

    let mut expect_syscall_exit = false;
    let mut current_ctx: Option<SyscallContext> = None;
    let mut sniffer_state = sniffer::SnifferState::new();

    loop {
        unsafe {
            ptrace(PTRACE_SYSCALL, pid, ptr::null_mut::<c_void>(), ptr::null_mut::<c_void>());
        }

        let mut status = 0;
        let res = unsafe { waitpid(pid, &mut status, 0) };
        if res == -1 { break; }

        if WIFEXITED(status) || WIFSIGNALED(status) {
            println!("Target exited.");
            break;
        }

        if WIFSTOPPED(status) {
            let sig = WSTOPSIG(status);
            if sig == (SIGTRAP | 0x80) {
                // Stopped at Syscall Entry or Exit
                if !expect_syscall_exit {
                    // SYSCALL ENTRY
                    // Check for injection
                    if let Ok(cmd) = rx.try_recv() {
                         match cmd {
                             listener::Command::ChannelOutputMute { fader_index, mute } => {
                                 println!("[Bridge] Preparing to inject Mute({}, {})", fader_index, mute);
                                  if let Ok(regs) = get_regs(pid) {
                                      let payload = protocol::build_channel_output_mute(fader_index, mute);
                                      let packet = protocol::Packet::new(payload);
                                      let bytes = packet.to_bytes();
                                      
                                      if let Err(e) = injector.inject(&bytes, &regs) {
                                           eprintln!("Injection failed: {}", e);
                                           // If injection fails, we just continue with original syscall
                                      } else {
                                           println!("[Bridge] Injection successful.");
                                           sniffer::print_hexdump("INJECTED", &bytes);
                                           // After injection, we are at restored entry state.
                                           // We must NOT check rx again immediately to avoid loop, 
                                           // and we should proceed to handle the original syscall entry below.
                                      }
                                  } else {
                                       eprintln!("Failed to get regs for injection");
                                  }
                             }
                        }
                    }

                    if let Ok(regs) = get_regs(pid) {
                        let sys_nr = regs.regs[8];
                        let arg_fd = regs.regs[0];
                        let is_target = arg_fd == target_fd;
                        
                        let is_relevant = is_target && (
                            sys_nr == SYS_READ || sys_nr == SYS_WRITE || sys_nr == SYS_READV || sys_nr == SYS_WRITEV || 
                            sys_nr == SYS_SENDTO || sys_nr == SYS_RECVFROM || sys_nr == SYS_SENDMSG || sys_nr == SYS_RECVMSG
                        );
                        
                        if is_relevant {
                            current_ctx = Some(SyscallContext {
                                syscall_nr: sys_nr,
                                fd: arg_fd,
                                buf_addr: regs.regs[1],
                                count: regs.regs[2],
                            });
                        } else {
                            current_ctx = None;
                        }
                    }
                    expect_syscall_exit = true;
                } else {
                    // SYSCALL EXIT
                    if let Some(ctx) = current_ctx.take() {
                        if let Ok(regs) = get_regs(pid) {
                            let ret_val = regs.regs[0] as i64;
                            if ret_val > 0 {
                                let direction = match ctx.syscall_nr {
                                    SYS_READ | SYS_READV | SYS_RECVFROM | SYS_RECVMSG => "READ",
                                    _ => "WRITE",
                                };

                                if ctx.syscall_nr == SYS_READV || ctx.syscall_nr == SYS_WRITEV {
                                     println!("{}: [IOVEC op: {} bytes]", direction, ret_val);
                                } else if ctx.syscall_nr == SYS_SENDMSG || ctx.syscall_nr == SYS_RECVMSG {
                                     println!("{}: [MSG op: {} bytes]", direction, ret_val);
                                } else {
                                     if let Ok(data) = read_memory(pid, ctx.buf_addr, ret_val as usize) {
                                        sniffer_state.handle_packet(direction, &data);
                                    }
                                }
                            }
                        }
                    }
                    expect_syscall_exit = false;
                }
            } else if sig != SIGTRAP {
                 unsafe { ptrace(PTRACE_SYSCALL, pid, ptr::null_mut::<c_void>(), sig as *mut c_void); }
                 continue;
            }
        }
    }
    
    unsafe { ptrace(PTRACE_DETACH, pid, ptr::null_mut::<c_void>(), ptr::null_mut::<c_void>()); }
    Ok(())
}

fn get_regs(pid: pid_t) -> Result<UserPtRegs, std::io::Error> {
    let mut regs: UserPtRegs = Default::default();
    let mut iov = iovec {
        iov_base: &mut regs as *mut _ as *mut c_void,
        iov_len: mem::size_of::<UserPtRegs>(),
    };
    let res = unsafe {
        ptrace(PTRACE_GETREGSET, pid, NT_PRSTATUS as *mut c_void, &mut iov as *mut _ as *mut c_void)
    };
    if res == -1 { Err(std::io::Error::last_os_error()) } else { Ok(regs) }
}

fn read_memory(pid: pid_t, addr: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
    let path = format!("/proc/{}/mem", pid);
    let mut file = std::fs::File::open(path)?;
    use std::os::unix::fs::FileExt;
    let mut buf = vec![0u8; len];
    file.read_at(&mut buf, addr)?;
    Ok(buf)
}
