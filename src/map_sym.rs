/// Symbol about memory map, 
/// provided by linker

extern "C" {
    pub fn sdata();
    pub fn edata();
    pub fn srodata();
    pub fn erodata();
    pub fn stext();
    pub fn etext();
    pub fn frames();
    pub fn sbss();
    pub fn ebss();
    pub fn skernel();
    pub fn ekernel();
}
