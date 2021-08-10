pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;

pub fn syscall(id: usize, param: [usize; 3]) -> isize{
    match id {
        SYS_WRITE => {
            sys_write(param[0], param[1] as *const u8, param[2])
        },
        SYS_EXIT => {
            println!("[kernel] syscall EXIT");
            0
        },
        _ => {
            panic!("No Implement syscall: {}", id);
            1
        }
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    const FD_STDOUT: usize = 1;
    match fd {
        FD_STDOUT => {
            let slice = unsafe {
                core::slice::from_raw_parts(buf, len) };
            let string = core::str::from_utf8(slice).unwrap();
            crate::println!("{}", string);
            0
        },
        _ => {
            panic!("Unsupport syscall");
        }
    }
}

pub fn sys_exit(xstate: usize) -> ! {
    crate::println!("[kernel] Application exit with code {}", xstate);
    panic!("");
}
