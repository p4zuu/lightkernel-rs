use x86_64::{registers::model_specific::Msr, PhysAddr, VirtAddr};

use crate::{cpu::msr::IA32_VMX_BASIC, mm::memory::virt_to_phys, virt::VirtError};

use super::asm::{asm_vmptrld, asm_vmread};

const _: () = assert!(core::mem::size_of::<VMCS>() == 0x1000);
const _: () = assert!(core::mem::align_of::<VMCS>() == 0x1000);

#[derive(Debug)]
#[repr(C, align(0x1000))]
pub struct VMCS {
    revision: u32,
    abort: u32,
    data: VMCSData,
}

impl VMCS {
    pub fn new() -> Self {
        Self {
            revision: 0,
            abort: 0,
            data: VMCSData::default(),
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
    pub fn vmptrld(&self) -> Result<(), VirtError> {
        // SAFETY: we rely on the borrow checker to validate that self is
        // always valid.
        unsafe { asm_vmptrld(self.paddr()?) }
    }

    fn init_revision(&mut self) {
        // SAFETY: Reading IA32_VMX_BASIC is safe
        let msr = unsafe { Msr::read(&Msr::new(IA32_VMX_BASIC)) };
        self.revision = msr as u32;
        self.revision &= !(1 << 31);
    }

    pub fn setup(&mut self) -> Result<(), VirtError> {
        self.init_revision();
        self.vmptrld()
    }

    pub fn vmread(&self, field: u32) -> Result<u32, VirtError> {
        // SAFETY: we rely on the borrow checker to validate that self is
        // always valid.
        unsafe { asm_vmread(field) }
    }

    pub fn is_shadow(&self) -> bool {
        self.revision & (1 << 31) != 0
    }

    pub fn set_shadow(&mut self) {
        self.revision &= 1 << 31
    }
}

impl Default for VMCS {
    fn default() -> Self {
        VMCS::new()
    }
}

#[derive(Debug)]
#[repr(C)]
struct VMCSData {
    data: [u8; 0x1000 - 8],
}

impl VMCSData {
    fn new() -> Self {
        Self {
            data: [0u8; 0x1000 - 8],
        }
    }
}

impl Default for VMCSData {
    fn default() -> Self {
        VMCSData::new()
    }
}
