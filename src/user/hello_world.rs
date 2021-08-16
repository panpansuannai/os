#[repr(align(2))]
#[link_section = ".user"]
pub extern "C" fn main() {
    let msg = "Hello world\n";
    let msg = ['h', 'e', 'l', 'l', 'o', 'w'];
    super::syscall(
        crate::syscall::SYS_WRITE, [1, msg.as_ptr() as usize, msg.len()]);
    let msg = "Happy to yielded\n";
    super::syscall(
        crate::syscall::SYS_WRITE, [1, msg.as_ptr() as usize, msg.len()]);
    super::syscall(
        crate::syscall::SYS_EXIT, [1,0,0]);
}
