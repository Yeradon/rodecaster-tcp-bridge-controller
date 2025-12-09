mod injector;
mod listener;
mod protocol;
mod sniffer;

use clap::Parser;
use libc::{
    c_void, iovec, pid_t, ptrace, waitpid, PTRACE_ATTACH, PTRACE_DETACH, PTRACE_GETREGSET,
    PTRACE_O_TRACESYSGOOD, PTRACE_SETOPTIONS, PTRACE_SYSCALL, SIGTRAP, WIFEXITED, WIFSIGNALED,
    WIFSTOPPED, WSTOPSIG,
};
use std::mem;
use std::ptr;
use std::sync::mpsc;

// AArch64 constants
const NT_PRSTATUS: i32 = 1;
const SYS_READ: u64 = 63;
const SYS_WRITE: u64 = 64;
const SYS_READV: u64 = 65;
const SYS_WRITEV: u64 = 66;
const SYS_SENDTO: u64 = 206;
const SYS_RECVFROM: u64 = 207;
const SYS_SENDMSG: u64 = 211;
const SYS_RECVMSG: u64 = 212;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct UserPtRegs {
    regs: [u64; 31],
    sp: u64,
    pc: u64,
    pstate: u64,
}

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

    // Prepare Injector (direct FD write)
    let mut injector = injector::Injector::new(pid, target_fd).map_err(|e| {
        eprintln!("Warning: Failed to open injection FD (maybe not yet open?): {}", e);
        e
    }).ok();

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
        // Before continuing, process any pending injection commands
        // We only inject when we are paused (which we are now).
        // Since we write to the FD directly, we don't need to hijack the current syscall.
        // We just need to ensure the process is stopped so we don't race too badly 
        // (though kernel handles socket write locking, doing it here is cleaner).
        
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                listener::Command::ChannelOutputMute { fader_index, mute } => {
                    println!("[Bridge] Injecting Mute(fader={}, mute={})", fader_index, mute);
                    let payload = protocol::build_channel_output_mute(fader_index, mute);
                    let packet = protocol::Packet::new(payload);
                    let bytes = packet.to_bytes();
                    
                    if let Some(inj) = &mut injector {
                        if let Err(e) = inj.inject(&bytes) {
                             eprintln!("[Bridge] Injection failed: {}", e);
                        } else {
                             println!("[Bridge] Injected {} bytes", bytes.len());
                             // Log it as outgoing
                             sniffer::print_hexdump("INJECTED", &bytes);
                        }
                    } else {
                        // Try to reopen if it failed initially
                        if let Ok(inj) = injector::Injector::new(pid, target_fd) {
                             injector = Some(inj);
                             // Retry once
                             if let Some(inj) = &mut injector {
                                 let _ = inj.inject(&bytes);
                                 sniffer::print_hexdump("INJECTED", &bytes);
                             }
                        } else {
                             eprintln!("[Bridge] Injection FD not available.");
                        }
                    }
                }
            }
        }

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
                if !expect_syscall_exit {
                    // SYSCALL ENTRY
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
