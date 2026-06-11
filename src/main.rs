#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


#[unsafe(no_mangle)] // Don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // kernel logic will begin here
    loop {}
}
