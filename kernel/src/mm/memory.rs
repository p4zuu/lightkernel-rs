use core::{alloc::Layout, ptr::NonNull};

use spinning_top::Spinlock;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable, Translate},
    PhysAddr, VirtAddr,
};

use super::alloc::{AllocError, PHYS_MEM_OFFSET};

// Safety: ROOT_MEM.page_range.{start, end} are note accessed before being initialized.
pub static ROOT_MEM: Spinlock<MemoryRegion> = Spinlock::new(MemoryRegion::empty());

pub const PAGE_SHIFT: usize = 12;

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Represents a memory region.
pub struct MemoryRegion {
    start_virt: VirtAddr,
    end_virt: VirtAddr,
    global_alloc: linked_list_allocator::Heap,
}

impl MemoryRegion {
    pub const fn empty() -> Self {
        Self {
            start_virt: VirtAddr::zero(),
            end_virt: VirtAddr::zero(),
            global_alloc: linked_list_allocator::Heap::empty(),
        }
    }

    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.start_virt = VirtAddr::new(heap_start as u64);
        self.end_virt = self.start_virt + heap_size as u64;

        unsafe {
            self.global_alloc.init(heap_start as *mut u8, heap_size);
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<VirtAddr, AllocError> {
        self.global_alloc
            .allocate_first_fit(layout)
            .map(|ptr| VirtAddr::from_ptr(ptr.as_ptr()))
            .map_err(|_| AllocError::OutOfMemory)
    }

    /// # Safety
    ///
    /// Caller should ensure linked_list_allocator::Heap::deallocate() safety
    /// requirements.
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.global_alloc
            .deallocate(NonNull::<u8>::new_unchecked(ptr), layout);
    }

    pub fn alloc_pages(&self, _order: u32) -> Result<VirtAddr, AllocError> {
        unimplemented!()
    }

    pub fn alloc_page(&self) -> Result<VirtAddr, AllocError> {
        self.alloc_pages(0)
    }
}

impl Default for MemoryRegion {
    fn default() -> Self {
        MemoryRegion::empty()
    }
}

pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    let mapper = unsafe {
        let pt = active_level_4_table(PHYS_MEM_OFFSET);
        OffsetPageTable::new(pt, PHYS_MEM_OFFSET)
    };

    mapper.translate_addr(addr)
}
