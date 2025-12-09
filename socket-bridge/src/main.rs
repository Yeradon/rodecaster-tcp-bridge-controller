use clap::Parser;
use libc::{
    c_void, iovec, pid_t, ptrace, waitpid, PTRACE_ATTACH, PTRACE_DETACH, PTRACE_GETREGSET,
    PTRACE_O_TRACESYSGOOD, PTRACE_SETOPTIONS, PTRACE_SYSCALL, SIGTRAP, WIFEXITED, WIFSIGNALED,
    WIFSTOPPED, WSTOPSIG,
};
use std::ffi::CString;
use std::mem;
use std::ptr;

// AArch64 constants
const NT_PRSTATUS: i32 = 1;
const SYS_READ: u64 = 63;
const SYS_WRITE: u64 = 64;
const SYS_READV: u64 = 65;
const SYS_WRITEV: u64 = 66;
// socket syscalls might be different on aarch64
// user_regs_struct on AArch64 uses x8 for syscall number.
// sendto = 206, recvfrom = 207, sendmsg = 211, recvmsg = 212
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

    println!("Attaching to PID: {}", pid);
    unsafe {
        let res = ptrace(
            PTRACE_ATTACH,
            pid,
            ptr::null_mut::<c_void>(),
            ptr::null_mut::<c_void>(),
        );
        if res == -1 {
            let err = std::io::Error::last_os_error();
            eprintln!("PTRACE_ATTACH failed: {}", err);
            return Err(err.into());
        }
        
        let mut status = 0;
        waitpid(pid, &mut status, 0);
        
        // Check if waitpid actually stopped specifically
        if !WIFSTOPPED(status) {
             eprintln!("Waitpid returned but process not stopped. Status: {}", status);
        }
        
        let res = ptrace(
            PTRACE_SETOPTIONS,
            pid,
            ptr::null_mut::<c_void>(),
            PTRACE_O_TRACESYSGOOD as *mut c_void,
        );
        if res == -1 {
            let err = std::io::Error::last_os_error();
            eprintln!("PTRACE_SETOPTIONS failed: {}", err);
            // Don't fail here, just warn
        }
    }
    println!("Attached. Sniffing FD: {}", target_fd);

    // Track if next stop is exit
    let mut expect_syscall_exit = false;
    // Store context from entry
    let mut current_ctx: Option<SyscallContext> = None;

    // Deduplication state
    let mut last_packet: Option<(String, Vec<u8>)> = None;
    let mut repeat_count: usize = 0;

    loop {
        unsafe {
            ptrace(
                PTRACE_SYSCALL,
                pid,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            );
        }

        let mut status = 0;
        let res = unsafe { waitpid(pid, &mut status, 0) };
        if res == -1 {
            break;
        }

        if WIFEXITED(status) || WIFSIGNALED(status) {
            println!("Target exited.");
            break;
        }

        if WIFSTOPPED(status) {
            let sig = WSTOPSIG(status);
            if sig == (SIGTRAP | 0x80) {
                // Syscall stop
                if !expect_syscall_exit {
                    // SYSCALL ENTRY
                    if let Ok(regs) = get_regs(pid) {
                        let sys_nr = regs.regs[8];
                        let arg_fd = regs.regs[0];
                        
                        let is_target = arg_fd == target_fd;
                        
                        // Check for all relevant syscalls
                        let is_relevant = is_target && (
                            sys_nr == SYS_READ || sys_nr == SYS_WRITE || 
                            sys_nr == SYS_READV || sys_nr == SYS_WRITEV || 
                            sys_nr == SYS_SENDTO || sys_nr == SYS_RECVFROM ||
                            sys_nr == SYS_SENDMSG || sys_nr == SYS_RECVMSG
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
                    } else {
                         current_ctx = None;
                    }
                    expect_syscall_exit = true;
                } else {
                    // SYSCALL EXIT
                    if let Some(ctx) = current_ctx.take() {
                        if let Ok(regs) = get_regs(pid) {
                            let ret_val = regs.regs[0] as i64; // x0 is return value
                            if ret_val > 0 {
                                let direction = match ctx.syscall_nr {
                                    SYS_READ | SYS_READV | SYS_RECVFROM | SYS_RECVMSG => "READ",
                                    _ => "WRITE",
                                };

                                if ctx.syscall_nr == SYS_READV || ctx.syscall_nr == SYS_WRITEV {
                                    if repeat_count > 0 {
                                        println!("(Previous packet repeated {} times)", repeat_count);
                                        repeat_count = 0;
                                        last_packet = None;
                                    }
                                    println!("{}: [IOVEC op: {} bytes]", direction, ret_val);
                                } else if ctx.syscall_nr == SYS_SENDMSG || ctx.syscall_nr == SYS_RECVMSG {
                                    if repeat_count > 0 {
                                        println!("(Previous packet repeated {} times)", repeat_count);
                                        repeat_count = 0;
                                        last_packet = None;
                                    }
                                    println!("{}: [MSG op: {} bytes]", direction, ret_val);
                                } else {
                                    // Normal buffer
                                     if let Ok(data) = read_memory(pid, ctx.buf_addr, ret_val as usize) {
                                        let current = (direction.to_string(), data.clone());
                                        if let Some((last_dir, last_data)) = &last_packet {
                                            if *last_dir == current.0 && *last_data == current.1 {
                                                repeat_count += 1;
                                            } else {
                                                if repeat_count > 0 {
                                                    println!("(Previous packet repeated {} times)", repeat_count);
                                                }
                                                print_hex(direction, &data);
                                                last_packet = Some(current);
                                                repeat_count = 0;
                                            }
                                        } else {
                                             print_hex(direction, &data);
                                             last_packet = Some(current);
                                             repeat_count = 0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    expect_syscall_exit = false;
                }
            } else if sig != SIGTRAP {
                 // Pass signal
                 unsafe {
                    ptrace(PTRACE_SYSCALL, pid, ptr::null_mut::<c_void>(), sig as *mut c_void);
                }
                continue;
            }
        }
    }
    
    // Detach cleanup
    unsafe {
        ptrace(PTRACE_DETACH, pid, ptr::null_mut::<c_void>(), ptr::null_mut::<c_void>());
    }

    Ok(())
}

fn get_regs(pid: pid_t) -> Result<UserPtRegs, std::io::Error> {
    let mut regs: UserPtRegs = Default::default();
    let mut iov = iovec {
        iov_base: &mut regs as *mut _ as *mut c_void,
        iov_len: mem::size_of::<UserPtRegs>(),
    };
    let res = unsafe {
        ptrace(
            PTRACE_GETREGSET,
            pid,
            NT_PRSTATUS as *mut c_void,
            &mut iov as *mut _ as *mut c_void,
        )
    };
    if res == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(regs)
    }
}

fn read_memory(pid: pid_t, addr: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
    // Process_vm_readv is cleaner but requires nix or correct libc usage. 
    // Using /proc/pid/mem is also an option, but let's stick to ptrace PEEKDATA loop or process_vm_readv if available.
    // Given we are in libc, process_vm_readv might be available.
    
    // Fallback to /proc/pid/mem for simplicity and speed if process_vm_readv logic is complex with raw pointers.
    // Actually, reading /proc/pid/mem is very robust.
    let path = format!("/proc/{}/mem", pid);
    let mut file = std::fs::File::open(path)?;
    use std::os::unix::fs::FileExt;
    let mut buf = vec![0u8; len];
    file.read_at(&mut buf, addr)?;
    Ok(buf)
}

fn print_hex(prefix: &str, data: &[u8]) {
    println!("{} ({} bytes):", prefix, data.len());
    let width = 16;
    for (i, chunk) in data.chunks(width).enumerate() {
        print!("{:08x}  ", i * width);
        
        // Hex part
        for (j, b) in chunk.iter().enumerate() {
            print!("{:02x} ", b);
            if j == 7 {
                print!(" ");
            }
        }
        
        // Padding if last chunk is short
        if chunk.len() < width {
            let missing = width - chunk.len();
            let spaces = missing * 3 + (if chunk.len() <= 8 { 1 } else { 0 });
            for _ in 0..spaces {
                print!(" ");
            }
        }
        
        print!(" |");
        // ASCII part
        for b in chunk {
            if *b >= 32 && *b <= 126 {
                print!("{}", *b as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
    println!();
}

