use crate::virt::{VMXResult, VirtError};
use core::arch::asm;
use x86_64::PhysAddr;

use super::errors::VM_INSTRUCTION_ERROR;

///
/// # Safety
///
/// Caller should ensure that the VMXON region is still allocated.
#[inline]
pub unsafe fn asm_vmxon(addr: PhysAddr) -> Result<(), VirtError> {
    let mut ret: u8;
    asm!(
        "vmxon [{addr}]; setna {ret}",
        addr = in(reg) &addr.as_u64(), ret = out(reg_byte) ret,
        options(readonly, nostack, preserves_flags)
    );

    match ret {
        0 => Ok(()),
        1 => Err(VirtError::VMInstruction(VMXResult::FailValid(asm_vmread(
            VM_INSTRUCTION_ERROR as u32,
        )?))),
        2 => Err(VirtError::VMInstruction(VMXResult::FailInvalid)),
        _ => unreachable!(),
    }
}

///
/// # Safety
///
/// Caller should ensure that the VMCS region is still allocated.
#[inline]
pub unsafe fn asm_vmptrld(addr: PhysAddr) -> Result<(), VirtError> {
    let mut ret: u8;
    asm!(
        "vmptrld [{addr}]; setna {ret}",
        addr = in(reg) &addr.as_u64(), ret = out(reg_byte) ret,
        options(readonly, nostack, preserves_flags)
    );

    match ret {
        0 => Ok(()),
        1 => Err(VirtError::VMInstruction(VMXResult::FailValid(asm_vmread(
            VM_INSTRUCTION_ERROR as u32,
        )?))),
        2 => Err(VirtError::VMInstruction(VMXResult::FailInvalid)),
        _ => unreachable!(),
    }
}

///
/// # Safety
///
/// Caller should ensure that the VMCS region is still allocated.
#[inline]
pub unsafe fn asm_vmread(field: u32) -> Result<u32, VirtError> {
    let mut ret: u8;
    let mut result: u32;
    asm!(
        "vmread [{result:r}], {field:r}; setna {ret}",
        result = out(reg) result, field = in(reg) field, ret = out(reg_byte) ret,
        options(readonly, nostack, preserves_flags)
    );

    match ret {
        0 => Ok(result),
        1 => Err(VirtError::VMInstruction(VMXResult::FailValid(asm_vmread(
            VM_INSTRUCTION_ERROR as u32,
        )?))),
        2 => Err(VirtError::VMInstruction(VMXResult::FailInvalid)),
        _ => unreachable!(),
    }
}
