pub const SET_TIMER : usize = 0;
pub const PUT_CHAR : usize = 1;
pub const GET_CHAR : usize = 2;
pub const CLEAR_IPI : usize = 3;
pub const SEND_IPI : usize = 4;
pub const REMOTE_FENCE_I : usize = 5;
pub const REMOTE_SFENCE_VMA : usize = 6;
pub const REMOTE_SFENCE_ASID : usize = 7;
pub const SHUTDOWN : usize = 8;

pub fn sbi_call(which: usize, args:[usize; 3]) -> usize {
    let mut ret;
    unsafe {
        llvm_asm!("ecall"
                 : "={x10}" (ret)
                 : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), 
                   "{x17}" (which)
                 : "memory"
                 : "volatile"
         );
    }
    ret
}

pub fn shutdown() -> usize{
    sbi_call(SHUTDOWN, [0,0,0])
}
