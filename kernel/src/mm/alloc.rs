use core::alloc::GlobalAlloc;
use core::ptr;

use super::block::FixedBlockAlloc;
use super::frame::BootInfoFrameAllocator;
use super::memory::{self, ROOT_MEM};
use bootloader_api::BootInfo;
use spinning_top::Spinlock;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 1000 * 1024;
pub static mut PHYS_MEM_OFFSET: VirtAddr = VirtAddr::zero();

#[global_allocator]
static ALLOCATOR: KernelAlloc = KernelAlloc::new();

/// Custom error for the allocator
#[derive(Debug)]
pub enum AllocError {
    /// Out of memory
    OutOfMemory,
    // Requested size is too big
    TooBig,
    /// The page allocator has no cached free pages
    PageAllocEmpty,
    // Requested layout is incorrect
    LayoutError,
}

/// Initialize the heap by mapping all pages.
pub fn init_mem(boot_info: &'static mut BootInfo) -> Result<(), MapToError<Size4KiB>> {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // SAFETY: the kernel is still single-threaded here.
    unsafe {
        PHYS_MEM_OFFSET = phys_mem_offset;
    }
    // SAFETY: TODO(tleroy)
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut frame_allocator)?
                .flush();
        }
    }

    ROOT_MEM.lock().init(HEAP_START, HEAP_SIZE);

    Ok(())
}

/// Global allocator for the kernel
struct KernelAlloc(Spinlock<FixedBlockAlloc>);

impl KernelAlloc {
    pub const fn new() -> Self {
        Self(Spinlock::new(FixedBlockAlloc::empty()))
    }
}

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let _size = layout.size();
        let _align = layout.align();

        // TODO(tleroy): check alignment and size
        self.0
            .lock()
            .alloc(layout)
            .map_or_else(|_| ptr::null_mut(), |addr| addr.as_mut_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.0.lock().dealloc(ptr, layout);
    }
}
