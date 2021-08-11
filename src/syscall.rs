pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_YIELD: usize = 124;

pub fn syscall(id: usize, param: [usize; 3]) -> isize{
    match id {
        SYS_WRITE => {
            sys_write(param[0], param[1] as *const u8, param[2])
        },
        SYS_EXIT => {
            sys_exit(param[0]);
        },
        SYS_YIELD => {
            sys_yield(param[0]);
        },
        _ => {
            panic!("No Implement syscall: {}", id);
        }
    }
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    const FD_STDOUT: usize = 1;
    match fd {
        FD_STDOUT => {
            let slice = unsafe {
                core::slice::from_raw_parts(buf, len) };
            let string = core::str::from_utf8(slice).unwrap();
            for c in string.chars() {
                crate::sbi::sbi_call(crate::sbi::PUT_CHAR, [c as usize, 0, 0]);
            }
            0
        },
        _ => {
            panic!("Unsupport syscall");
        }
    }
}

fn sys_exit(xstate: usize) -> ! {
    crate::println!("[kernel] Application exit with code {}", xstate);
    use crate::task::TASK_MANAGER;
    TASK_MANAGER.exit_current_task();
    TASK_MANAGER.start_next_task();
    panic!("");
}

fn sys_yield(cx: usize) -> ! {
    println!("[kernel] syscall Yield");
    use crate::task::TASK_MANAGER;
    TASK_MANAGER.set_current_task_cx(cx);
    TASK_MANAGER.set_current_task_ready();
    TASK_MANAGER.start_next_task();
    println!("[kernel] syscall Yield unreachable");
    loop {}
}
