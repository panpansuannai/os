use crate::{mm::address::*, console::print, process::cpu::current_hart, task::{schedule_pcb, TASKMANAGER}};
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
            sys_yield(param[0])
        },
        _ => {
            panic!("No Implement syscall: {}", id);
        }
    }
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let mut buffer = alloc::vec![0_u8; len];
    if TASKMANAGER.is_locked() {
        panic!("sys_write still locking");
    }
    TASKMANAGER.lock().current_pcb().lock().memory_space.copy_virtual_address(VirtualAddr(buf as usize), len, buffer.as_mut_slice());
    const FD_STDOUT: usize = 1;
    match fd {
        FD_STDOUT => {
            let slice = buffer.as_slice();
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
    TASKMANAGER.lock().current_pcb().lock().exit();
    schedule_pcb();
}

fn sys_yield(_: usize) -> isize {
    println!("[kernel] syscall Yield");
    TASKMANAGER.lock().current_pcb().lock().set_state(crate::process::PcbState::Ready);
    0
}
