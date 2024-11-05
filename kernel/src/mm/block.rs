use core::alloc::Layout;
use core::ptr::{addr_of, addr_of_mut};
use core::{fmt, mem};

use x86_64::VirtAddr;

use super::alloc::AllocError;
use super::memory::ROOT_MEM;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Node for the block linked-list
#[repr(transparent)]
struct BlockHeader {
    next: Option<&'static mut Block>,
}

impl BlockHeader {
    fn new(next: Option<&'static mut Block>) -> Self {
        Self { next }
    }
}

impl fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.next)
    }
}

#[repr(transparent)]
struct Block {
    header: BlockHeader,
}

impl Block {
    fn new(header: BlockHeader) -> Self {
        Self { header }
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self as *const Self, self.header)
    }
}

/// Fixed-size block allocator
pub struct FixedBlockAlloc {
    block_lists: [Option<&'static mut Block>; BLOCK_SIZES.len()],
}

impl FixedBlockAlloc {
    pub const fn empty() -> Self {
        const EMPTY: Option<&'static mut Block> = None;
        Self {
            block_lists: [EMPTY; BLOCK_SIZES.len()],
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<VirtAddr, AllocError> {
        match list_index(layout) {
            Some(index) => match self.block_lists[index].take() {
                Some(node) => {
                    self.block_lists[index] = node.header.next.take();
                    Ok(VirtAddr::from_ptr(addr_of!(*node)))
                }
                None => {
                    let block_size = BLOCK_SIZES[index];
                    let block_align = block_size;
                    let layout = Layout::from_size_align(block_size, block_align)
                        .map_err(|_| AllocError::LayoutError)?;
                    self.fallback_alloc(layout)
                }
            },
            None => ROOT_MEM.lock().alloc(layout),
        }
    }

    fn fallback_alloc(&mut self, layout: Layout) -> Result<VirtAddr, AllocError> {
        // Find first non-empty list for bigger blocks
        let Some(upper_index) = self.first_list_with_block(layout) else {
            return ROOT_MEM.lock().alloc(layout);
        };

        if layout.size() <= BLOCK_SIZES[upper_index - 1] || layout.size() > BLOCK_SIZES[upper_index]
        {
            self.split_and_downgrade(upper_index)?;
            return self.fallback_alloc(layout);
        }

        let node = self.pop_head(upper_index).ok_or(AllocError::LayoutError)?;
        Ok(VirtAddr::from_ptr(addr_of!(*node)))
    }

    fn split_head(
        &mut self,
        index: usize,
    ) -> Result<(&'static mut Block, &'static mut Block), AllocError> {
        // unlink the first free node
        let head = self.pop_head(index).ok_or(AllocError::LayoutError)?;

        let new_node_ptr = addr_of_mut!(*head).wrapping_byte_add(BLOCK_SIZES[index - 1]);
        // SAFETY: we trust the previous memory block to be valid. Therefore, the address
        // of the middle of the block is valid. This address will be the address
        // of the new block. We also trust the allocator to ensure that a block never
        // overlaps to other valid block, so the second half of the block never overlaps
        // with something else.
        // Finally, we'll initialize the new second block afterwards. This will create
        // a valid reference.
        let new_node = unsafe { &mut *new_node_ptr };

        Ok((head, new_node))
    }

    fn split_and_downgrade(&mut self, index: usize) -> Result<(), AllocError> {
        assert!(index > 0);
        assert!(index < BLOCK_SIZES[BLOCK_SIZES.len() - 1]);

        let (node1, node2) = self.split_head(index)?;

        self.put_head(index - 1, node2);
        self.put_head(index - 1, node1);

        Ok(())
    }

    /// Puts node as current head of the list corresponding to index.
    fn put_head(&mut self, index: usize, node: &'static mut Block) {
        node.header.next = self.block_lists[index].take();
        self.block_lists[index] = Some(node);
    }

    /// Pops the current head of the list corresponding to index.
    /// Sets current head's next node as new current head.
    fn pop_head(&mut self, index: usize) -> Option<&'static mut Block> {
        let node = self.block_lists[index].take()?;
        self.block_lists[index] = node.header.next.take();
        Some(node)
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        match list_index(layout) {
            Some(index) => {
                assert!(mem::size_of::<BlockHeader>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<BlockHeader>() <= BLOCK_SIZES[index]);

                let current_top = Block::new(BlockHeader::new(self.block_lists[index].take()));
                let new_node_ptr = addr_of_mut!(*ptr).cast::<Block>();
                new_node_ptr.write(current_top);
                self.block_lists[index] = Some(&mut *new_node_ptr);
            }
            None => ROOT_MEM.lock().dealloc(ptr, layout),
        }
    }

    /// Returns the first list containing at least one available block.
    pub fn first_list_with_block(&self, layout: Layout) -> Option<usize> {
        let block_size = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&s| {
            if let Some(index) = size_to_index(s) {
                return s >= block_size && self.block_lists[index].is_some();
            }

            false
        })
    }
}

impl fmt::Debug for FixedBlockAlloc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FixedBlockAlloc:")?;
        for (i, b) in self.block_lists.iter().enumerate().rev() {
            writeln!(f, "\t{}\t-> {:?}", BLOCK_SIZES[i], b)?;
        }
        Ok(())
    }
}

/// Returns the index of BLOCK_SIZES related to the correct block size.
pub fn list_index(layout: Layout) -> Option<usize> {
    let block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= block_size)
}

fn size_to_index(size: usize) -> Option<usize> {
    let res = (size.ilog2() - BLOCK_SIZES[0].ilog2()) as usize;
    if res > 8 {
        return None;
    }

    Some(res)
}
