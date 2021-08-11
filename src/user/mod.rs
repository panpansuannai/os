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
        asm!("ecall", inout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id);
    }
    ret
}
