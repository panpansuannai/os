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
        asm!("ecall", inout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") which);
    }
    ret
}

pub fn shutdown() -> usize{
    sbi_call(SHUTDOWN, [0,0,0])
}
