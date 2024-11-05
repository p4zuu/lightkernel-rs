pub mod vmx;

#[derive(Debug)]
pub enum VMXResult {
    Succeed,
    FailValid(u32),
    FailInvalid,
}

#[derive(Debug)]
pub enum VirtError {
    /// Bad address requested.
    BadAddress(u64),
    /// Error while executing a VMX instruction.
    VMInstruction(VMXResult),
}
