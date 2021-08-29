pub const APP_NUM: usize = 1;

global_asm!(include_str!("apps.s"));
extern "C" {
    fn hello();
    fn hello_end();
}

lazy_static!{
pub static ref APP_START : [(usize, usize); APP_NUM] = 
    [(hello as usize, hello_end as usize)];
}

