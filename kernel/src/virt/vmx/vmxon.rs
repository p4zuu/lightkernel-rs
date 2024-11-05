use x86_64::{
    registers::{
        control::{Cr0, Cr0Flags, Cr4, Cr4Flags},
        model_specific::Msr,
    },
    PhysAddr, VirtAddr,
};

use super::asm::asm_vmxon;
use crate::virt::VirtError;
use crate::{
    cpu::msr::{
        IA32_VMX_BASIC, IA32_VMX_CR0_FIXED0, IA32_VMX_CR0_FIXED1, IA32_VMX_CR4_FIXED0,
        IA32_VMX_CR4_FIXED1,
    },
    mm::memory::virt_to_phys,
};

const _: () = assert!(core::mem::size_of::<VmxOn>() == 0x1000);
const _: () = assert!(core::mem::align_of::<VmxOn>() == 0x1000);

#[derive(Debug)]
#[repr(C, align(0x1000))]
pub struct VmxOn {
    revision: u32,
    _pad: [u8; 0x1000 - 4],
}

impl VmxOn {
    pub fn new() -> Self {
        Self {
            revision: 0,
            _pad: [0u8; 0x1000 - 4],
        }
    }

    #[inline]
    pub fn vaddr(&self) -> VirtAddr {
        VirtAddr::from_ptr(self as *const Self)
    }

    #[inline]
    pub fn paddr(&self) -> Result<PhysAddr, VirtError> {
        virt_to_phys(self.vaddr()).ok_or(VirtError::BadAddress(self.vaddr().as_u64()))
    }

    #[inline]
    pub fn vmxon(&self) -> Result<(), VirtError> {
        // SAFETY: we rely on the borrow checker to validate that self is
        // always valid.
        unsafe { asm_vmxon(self.paddr()?) }
    }

    pub fn enable_vmxe(&self) {
        unsafe {
            Cr0::update(|cr0| cr0.set(Cr0Flags::PROTECTED_MODE_ENABLE, true));
            Cr4::update(|cr4| cr4.set(Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS, true));

            let msr_cr0_0 = Msr::read(&Msr::new(IA32_VMX_CR0_FIXED0));
            let msr_cr0_1 = Msr::read(&Msr::new(IA32_VMX_CR0_FIXED1));
            let msr_cr4_0 = Msr::read(&Msr::new(IA32_VMX_CR4_FIXED0));
            let msr_cr4_1 = Msr::read(&Msr::new(IA32_VMX_CR4_FIXED1));

            Cr0::update(|cr0| {
                *cr0 |= Cr0Flags::from_bits_truncate(msr_cr0_0)
                    & Cr0Flags::from_bits_truncate(msr_cr0_1)
            });
            Cr4::update(|cr4| {
                *cr4 |= Cr4Flags::from_bits_truncate(msr_cr4_0)
                    & Cr4Flags::from_bits_truncate(msr_cr4_1)
            });
        }
    }

    fn init_revision(&mut self) {
        // SAFETY: Reading IA32_VMX_BASIC is safe
        let msr = unsafe { Msr::read(&Msr::new(IA32_VMX_BASIC)) };
        self.revision = msr as u32;
        self.revision &= !(1 << 31);
    }

    pub fn setup(&mut self) -> Result<(), VirtError> {
        self.enable_vmxe();
        self.init_revision();
        self.vmxon()
    }
}

impl Default for VmxOn {
    fn default() -> Self {
        VmxOn::new()
    }
}
