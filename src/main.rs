#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


#[no_mangle] // Don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // kernel logic will begin here
    loop {}
}
