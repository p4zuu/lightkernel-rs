use core::arch::asm;
use core::fmt;

pub struct IOPort(u16);

impl IOPort {
    pub const fn new(port: u16) -> Self {
        Self(port)
    }
}

impl fmt::Write for IOPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            // SAFETY: The caller should ensure writing to this port won't cause UB.
            unsafe {
                self.outb(b);
            }
        }

        Ok(())
    }
}

impl IOPort {
    pub unsafe fn outb(&self, value: u8) {
        unsafe { outb(self.0, value) }
    }
}

#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        // https://github.com/rust-osdev/x86_64/blob/master/src/instructions/port.rs
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}
