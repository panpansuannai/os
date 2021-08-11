use core::cell::RefCell;
use core::marker::Sync;
use crate::trap::{
    context::TrapContext,
    self
};


const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub static KERNEL_STACK: KernelStack = KernelStack { data: [0; KERNEL_STACK_SIZE] };
pub static USER_STACK0: UserStack = UserStack { data: [0; USER_STACK_SIZE] };
pub static USER_STACK1: UserStack = UserStack { data: [0; USER_STACK_SIZE] };

pub const APP_BASE_ADDR : usize = 0x80300000;

#[repr(align(4096))]
pub struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }

    pub fn push_context(&self, cx: TrapContext) -> &mut TrapContext {
        println!("[kernel] Pushing context");
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe { *cx_ptr = cx; }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

#[repr(align(4096))]
pub struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub struct AppManager {
    pub inner: RefCell<AppManagerInner>
}

pub struct AppManagerInner {
    pub num_app: usize,
    pub current_app: usize,
    pub app_start: [(usize,usize); crate::user::APP_NUM]
}

// global_asm!(include_str!("trap/traps.s"));

impl AppManagerInner {
    pub fn run_app(&self, app_id: usize){
        // For now, don't need load app
        // self.load_app(app_id);
        println!("[kernel] App loaded");
        crate::trap::_restore(KERNEL_STACK.push_context(
                    TrapContext::app_init_context(
                        self.app_start_addr(app_id).unwrap(),
                        USER_STACK0.get_sp(), 
                        0, KERNEL_STACK.get_sp(), 0)) 
                        as *const TrapContext as usize);
    }
    pub fn load_app(&self, app_id: usize){
        if app_id > crate::user::APP_NUM {
            panic!("Can't load app");
        }
        println!("[kernel] Loading app");
        unsafe { asm!("fence.i") }
        let app = self.app_start[app_id];
        let app_src = unsafe { 
            core::slice::from_raw_parts(app.0 as *const u8, app.1 - app.0)
        };
        let app_dst = unsafe {
            core::slice::from_raw_parts_mut(APP_BASE_ADDR as *mut u8, app_src.len())
        };
        app_dst.copy_from_slice(app_src);
    }

    pub fn clear_app(&self, app_id: usize) {
        let app_dst = unsafe { 
            core::slice::from_raw_parts_mut(self.app_start_addr(app_id).unwrap() as *mut u8,
                                          self.app_end_addr(app_id).unwrap()
                                          - self.app_start_addr(app_id).unwrap())
        };
        for u in app_dst {
            *u = 0;
        }
                                                     
    }
    pub fn app_start_addr(&self, app_id: usize) -> Option<usize> {
        match app_id {
            0..=crate::user::APP_NUM => {
                Some(self.app_start[app_id].0)
            },
            _ => {
                None
            }
        }
    }
    pub fn app_end_addr(&self, app_id: usize) -> Option<usize> {
        match app_id {
            0..=crate::user::APP_NUM => {
                Some(self.app_start[self.current_app].1)
            },
            _ => {
                None
            }
        }
    }
}

unsafe impl Sync for AppManager{}

lazy_static!{
pub static ref APP_MANAGER : AppManager = AppManager {
        inner : RefCell::new({
            AppManagerInner {
                num_app : crate::user::APP_NUM,
                current_app: 0,
                app_start : *crate::user::APP_START
            }
        }),
};
}

pub fn init(){
    return ;
    use crate::map_sym::{boot_stack, boot_stack_top};
    let stack_len = boot_stack_top as usize - boot_stack as usize;

    let stack_dst = unsafe { 
        core::slice::from_raw_parts_mut(
            KERNEL_STACK.data.as_ptr() as *mut u8, stack_len)
    };

    let sp_ori = unsafe {
        core::slice::from_raw_parts(boot_stack as *const u8, stack_len)};
    let mut j = 10;
    for i in sp_ori.iter() {
        j -= 1;
        println!("{}", *i);
        if j <= 0 {
            break;
        }
    }
}
