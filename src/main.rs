#![no_std]
#![no_main]
#![feature(panic_implementation)]

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_implementation]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}