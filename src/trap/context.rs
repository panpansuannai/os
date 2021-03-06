use riscv::register::sstatus::{Sstatus, self, SPP};

#[repr(C)]
#[derive(Debug)]
pub struct TrapContext {
    pub general_reg: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub satp: usize,

    // Read only
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

impl TrapContext {
    fn set_sp(&mut self, sp : usize) {
        self.general_reg[2] = sp;
    }
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        satp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        println!("[kernel] App initialize context");
        let mut sstatus = sstatus::read();
        // set CPU privilege to User after trapping back
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            general_reg: [0; 32],
            sstatus,
            sepc: entry,
            satp,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp);
        cx
    }
}
