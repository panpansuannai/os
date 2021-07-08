/// Symbol about memory map, 
/// provided by linker

extern "C" {
    pub fn sdata();
    pub fn edata();
    pub fn srodata();
    pub fn erodata();
    pub fn stext();
    pub fn etext();
    pub fn sbss();
    pub fn ebss();
}
