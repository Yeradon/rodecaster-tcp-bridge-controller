
// Common definitions shared across modules

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct UserPtRegs {
    pub regs: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
}
