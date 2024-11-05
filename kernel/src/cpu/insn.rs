use core::arch::asm;

pub enum Segment {
    CS(u64),
    DS(u64),
    ES(u64),
    SS(u64),
    FS(u64),
    GS(u64),
}

impl Segment {
    #[inline]
    pub fn read(self) -> u64 {
        let mut ret: u64;
        match self {
            Self::CS(_) => unsafe { asm!("mov {ret}, cs", ret = out(reg) ret) },
            _ => unimplemented!(),
        }
        ret
    }
}
