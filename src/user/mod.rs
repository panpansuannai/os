pub mod play;

pub const APP_NUM: usize = 3;

global_asm!(include_str!("apps.s"));
extern "C" {
    fn hello();
    fn hello_end();
}

lazy_static!{
pub static ref APP_START : [(usize, usize); APP_NUM] = 
    [(hello as usize, hello_end as usize),
    (play::main as usize, play::main as usize + 2048), (0, 0)
    ];
}

#[link_section=".user"]
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
