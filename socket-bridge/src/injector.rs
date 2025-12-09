use libc::{
    c_void, iovec, pid_t, ptrace, waitpid, PTRACE_GETREGS, PTRACE_SETREGS, PTRACE_SYSCALL,
    PTRACE_CONT, PTRACE_SETOPTIONS, PTRACE_O_TRACESYSGOOD, SIGTRAP, WIFSTOPPED, WSTOPSIG, NT_PRSTATUS
};
use std::mem;
use std::ptr;
use std::io::Write;
use crate::common::UserPtRegs;

pub struct Injector {
    pid: pid_t,
    fd: u64,
}

impl Injector {
    pub fn new(pid: pid_t, fd: u64) -> Self {
        Injector { pid, fd }
    }

    pub fn inject(&self, data: &[u8], orig_regs: &UserPtRegs) -> Result<(), String> {
        unsafe {
            // 1. Write payload to stack (Red zone safety: sp - 512)
            let stack_addr = orig_regs.sp - 512;
            self.write_memory(stack_addr, data).map_err(|e| format!("Mem write failed: {}", e))?;

            // 2. Prepare regs for write(fd, stack_addr, len)
            // AArch64 Syscalls: x8 = sys_nr, x0..x7 args
            // WRITE = 64
            let mut new_regs = *orig_regs;
            new_regs.regs[8] = 64; // SYS_WRITE
            new_regs.regs[0] = self.fd;
            new_regs.regs[1] = stack_addr;
            new_regs.regs[2] = data.len() as u64;
            
            // Set PC to current PC (which is at SVC instruction usually?)
            // If we are at syscall entry, the kernel has paused us.
            // When we resume with PTRACE_SYSCALL, it will execute the syscall defined in regs.
            
            self.set_regs(&new_regs).map_err(|e| format!("SetRegs failed: {}", e))?;

            // 3. Execute the injected syscall
            ptrace(PTRACE_SYSCALL, self.pid, ptr::null_mut::<c_void>(), ptr::null_mut::<c_void>());
            
            // 4. Wait for syscall exit
            let mut status = 0;
            waitpid(self.pid, &mut status, 0);
            
            // TODO: verify stop signal?
            
            // 5. Restore original regs
            // IMPORTANT: We must rewind PC to retry the original syscall.
            // On AArch64, the PC advances after the syscall instruction normally?
            // Actually, if we are at syscall entry, PC points to the instruction *after* SVC?
            // Or *at* SVC? 
            // In Linux AArch64 ptrace:
            // "The PC register points to the instruction following the SVC instruction"
            // So we need to rewind 4 bytes to execute SVC again.
            
            let mut restore_regs = *orig_regs;
            restore_regs.pc -= 4; // Rewind to SVC
            
            self.set_regs(&restore_regs).map_err(|e| format!("RestoreRegs failed: {}", e))?;
            
            Ok(())
        }
    }

    unsafe fn set_regs(&self, regs: &UserPtRegs) -> Result<(), std::io::Error> {
        let mut iov = iovec {
            iov_base: regs as *const _ as *mut c_void,
            iov_len: mem::size_of::<UserPtRegs>(),
        };
        let res = ptrace(
            libc::PTRACE_SETREGSET, // Note: Use PTRACE_SETREGSET with NT_PRSTATUS
            self.pid,
            NT_PRSTATUS as *mut c_void,
            &mut iov as *mut _ as *mut c_void,
        );
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn write_memory(&self, addr: u64, data: &[u8]) -> std::io::Result<()> {
        let path = format!("/proc/{}/mem", self.pid);
        let mut file = std::fs::OpenOptions::new().write(true).read(true).open(path)?;
        use std::os::unix::fs::FileExt;
        file.write_at(data, addr)?;
        Ok(())
    }
}
