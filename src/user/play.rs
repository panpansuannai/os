#[repr(align(2))]
#[link_section = ".user"]
pub extern "C" fn main() {
    let msg = "I'm a naughty boy\n";
    super::syscall(
        crate::syscall::SYS_WRITE, [1, msg.as_ptr() as usize, msg.len()]);
    let msg = "But I would still yield\n";
    super::syscall(
        crate::syscall::SYS_WRITE, [1, msg.as_ptr() as usize, msg.len()]);
    super::syscall(
        crate::syscall::SYS_EXIT, [0;3]);
}
