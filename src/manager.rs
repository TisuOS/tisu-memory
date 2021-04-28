//! # 内存管理器
//! 处理内存请求（页面、堆内存），这是此库提供的默认实现实现
//! ## 使用示例
//! ```rust
//! let mgr = MemoryManager::new(0, 128, 4096, 0x100000);
//! let addr = mgr.alloc_memory(4, true);
//! *addr = 9;
//! mgr.free_kernel_memory(addr);
//! ```
//! 
//! 2021年4月14日 zg

use tisu_sync::ContentMutex;
use crate::{MemoryOp, require::{HeapOp, PageOp}};

pub struct MemoryManager<T1 : PageOp, T2 : HeapOp<T1>> {
    kernel_start : *mut u8,
    user_start : *mut u8,
    page : ContentMutex<T1>,
    memory : ContentMutex<T2>,
}

impl<T1 : PageOp, T2 : HeapOp<T1>> MemoryManager<T1, T2> {
    pub fn new(
        heap_start : usize,
        kernel_page_num : usize,
        page_size : usize,
        memory_end : usize
    )->Self {
        let user_heap = heap_start + kernel_page_num * page_size;
        let page = T1::new(heap_start,
            user_heap, memory_end, page_size);
        let p = page.clone();
        Self {
            kernel_start : heap_start as *mut u8,
            user_start : user_heap as *mut u8,
            page: ContentMutex::new(page),
            memory: ContentMutex::new(T2::new(p)),
        }
    }

}

impl<T1 : PageOp, T2 : HeapOp<T1>> MemoryOp for MemoryManager<T1, T2> {
    fn kernel_page(&mut self, num : usize)->Option<*mut u8> {
        let rt = self.page.lock().alloc_kernel_page(num).unwrap();
        assert!(rt as usize != 0x200_0000);
        Some(rt)
    }

    fn user_page(&mut self, num : usize)->Option<*mut u8> {
        self.page.lock().alloc_user_page(num)
    }

    fn free_page(&mut self, addr : *mut u8) {
        self.page.lock().free_page(addr);
    }

    fn alloc_memory(&mut self, size : usize, is_kernel : bool)->Option<*mut u8> {
        let mut memory = self.memory.lock();
        if is_kernel {
            (*memory).alloc_kernel_memory(size)
        }
        else {
            (*memory).alloc_user_memory(size)
        }
    }

    fn free_memory(&mut self, addr : *mut u8) {
        if addr >= self.kernel_start && addr < self.user_start {
            self.memory.lock().free_kernel_memory(addr);
        }
        else if addr >= self.user_start {
            self.memory.lock().free_user_memory(addr);
        }
        else {
            panic!("free memory error addr {:x}", addr as usize);
        }
    }

    fn print(&mut self) {
        self.page.lock().print();
        self.memory.lock().print();
    }
}