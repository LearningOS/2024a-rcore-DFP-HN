//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.
mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker, FRAME_ALLOCATOR};
pub use memory_set::remap_test;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_str, PageTableEntry, translate_virt_phy};
use page_table::{PTEFlags, PageTable};
/// 初始化内存子系统
///
/// 步骤：
/// 1) 初始化内核堆分配器（供 `alloc` 使用）
/// 2) 初始化物理页帧分配器（可分配区间为 `[ekernel, MEMORY_END)`）
/// 3) 激活内核地址空间 `KERNEL_SPACE`（写入 `satp` 并使用 SV39 映射）
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
