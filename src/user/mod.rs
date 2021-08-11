mod hello_world;
mod play;

pub const APP_NUM: usize = 3;

lazy_static!{
pub static ref APP_START : [(usize, usize); APP_NUM] = 
    [(hello_world::main as *const () as usize, hello_world::main as usize + 2048), (play::main as *const () as usize, play::main as usize + 2048), (0, 0)
    ];
}

fn syscall(id: usize, args: [usize; 3]) -> usize{
    let mut ret :usize ;
    unsafe {
        llvm_asm!("ecall"
                 : "={x10}" (ret)
                 : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), 
                   "{x17}" (id)
                 : "memory"
                 : "volatile"
         );
    }
    ret
}
