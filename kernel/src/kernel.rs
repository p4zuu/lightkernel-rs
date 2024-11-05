#![no_main]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;
#[allow(unused)]
use bootloader_api::{
    config, config::Mapping, entry_point, info::Optional, BootInfo, BootloaderConfig,
};
use kernel::cpu::idt::init_early_idt;
use kernel::logger::init_logger;
use kernel::mm::alloc::init_mem;
use kernel::virt::vmx::vmcs::VMCS;
use kernel::virt::vmx::vmxon::VmxOn;

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};
bootloader_api::entry_point!(kernel_main, config = &CONFIG);

#[no_mangle]
pub fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    init_logger().expect("failed to init logger");
    init_early_idt();
    init_mem(boot_info).expect("failed to init the kernel heap");

    let mut vmxon = Box::new(VmxOn::new());
    vmxon.setup().expect("failed to load vmxon region");

    let mut vmcs = Box::new(VMCS::new());
    vmcs.setup().expect("failed to load VMCS region");

    if let Err(e) = vmcs.vmread(0) {
        panic!("Failed to vmread. VM-instruction error: {:?}", e);
    }

    log::info!("Entering kernel loop");
    loop {}
}
