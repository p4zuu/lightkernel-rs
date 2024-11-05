use spin::Lazy;
use x86_64::{registers::control::Cr2, structures::idt::*};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.page_fault.set_handler_fn(pf_handler);
    idt.invalid_opcode.set_handler_fn(ud_handler);
    idt.general_protection_fault.set_handler_fn(gp_handler);
    idt
});

extern "x86-interrupt" fn pf_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "Unhandled #PF happend - RIP: {:#018x}, errror code: {:#018x}, address: {:#018x}",
        stack_frame.instruction_pointer,
        error_code,
        Cr2::read().unwrap()
    );
}

extern "x86-interrupt" fn ud_handler(stack_frame: InterruptStackFrame) {
    panic!(
        "Unhandled #UD happend - RIP: {:#018x}",
        stack_frame.instruction_pointer,
    );
}

extern "x86-interrupt" fn gp_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "Unhandled #GP happend - RIP: {:#018x}, error code: {:#018x}",
        stack_frame.instruction_pointer, error_code
    );
}

pub fn init_early_idt() {
    IDT.load();
}
