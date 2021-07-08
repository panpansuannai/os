/// core library doesn't provide us 
/// a panic_handler, we need one to 
/// handle panic 
use core::panic::PanicInfo;
use crate::println;

#[panic_handler]
fn panic(info : &PanicInfo) -> ! {
     if let Some(location) = info.location() {
        println!("Panicked at {}:{} {}", location.file(),
            location.line(), info.message().unwrap());
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    crate::sbi::shutdown();
    loop {}
}
