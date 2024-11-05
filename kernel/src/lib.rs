#![no_main]
#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::arch::asm;
use core::panic::PanicInfo;

pub mod cpu;
pub mod io;
pub mod logger;
pub mod mm;
pub mod virt;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("Panic: {}", info);
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
