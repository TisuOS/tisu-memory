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

use tisu_sync::SpinMutex;
use crate::{MemoryOp, require::{HeapOp, PageOp}};

pub struct MemoryManager<T1 : PageOp, T2 : HeapOp<T1>> {
    kernel_start : *mut u8,
    user_start : *mut u8,
    page : T1,
    memory : T2,
    kernel_mutex : SpinMutex,
    user_mutex : SpinMutex,
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
            page,
            memory: T2::new(p),
            kernel_mutex : SpinMutex::new(),
            user_mutex : SpinMutex::new(),
        }
    }

}

impl<T1 : PageOp, T2 : HeapOp<T1>> MemoryOp for MemoryManager<T1, T2> {
    fn kernel_page(&mut self, num : usize)->Option<*mut u8> {
        self.kernel_mutex.lock_no_int();
        let rt = self.page.alloc_kernel_page(num).unwrap();
        self.kernel_mutex.unlock_no_int();
        Some(rt)
    }

    fn user_page(&mut self, num : usize)->Option<*mut u8> {
        self.user_mutex.lock_no_int();
        let rt = self.page.alloc_user_page(num);
        self.user_mutex.unlock_no_int();
        rt
    }

    fn free_page(&mut self, addr : *mut u8) {
        if addr as usize >= self.user_start as usize {
            self.user_mutex.lock_no_int();
            self.page.free_page(addr);
            self.user_mutex.unlock_no_int();
        }
        else {
            self.kernel_mutex.lock_no_int();
            self.page.free_page(addr);
            self.kernel_mutex.unlock_no_int();
        }
    }

    fn alloc_memory(&mut self, size : usize, is_kernel : bool)->Option<*mut u8> {
        let rt;
        if is_kernel {
            self.kernel_mutex.lock_no_int();
            rt = self.memory.alloc_kernel_memory(size);
            self.kernel_mutex.unlock_no_int();
        }
        else {
            self.user_mutex.lock_no_int();
            rt = self.memory.alloc_user_memory(size);
            self.user_mutex.unlock_no_int();
        }
        rt
    }

    fn free_memory(&mut self, addr : *mut u8) {
        if addr >= self.kernel_start && addr < self.user_start {
            self.kernel_mutex.lock_no_int();
            self.memory.free_kernel_memory(addr);
            self.kernel_mutex.unlock_no_int();
        }
        else if addr >= self.user_start {
            self.user_mutex.lock_no_int();
            self.memory.free_user_memory(addr);
            self.user_mutex.unlock_no_int();
        }
        else {
            panic!("free memory error addr {:x}", addr as usize);
        }
    }

    fn print(&mut self) {
        self.kernel_mutex.lock_no_int();
        self.page.print();
        self.memory.print();
        self.kernel_mutex.unlock_no_int();
    }
}