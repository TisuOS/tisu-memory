//! # 内存管理器
//! 处理内存请求（页面、堆内存），这是此库提供的实现
//! 
//! 2021年4月14日 zg

use tisu_sync::ContentMutex;
use crate::{MemoryOp, require::{HeapOp, PageOp}};

pub struct MemoryManager<T1 : PageOp, T2 : HeapOp<T1>> {
    page : ContentMutex<T1>,
    memory : ContentMutex<T2>,
}

impl<T1 : PageOp, T2 : HeapOp<T1>> MemoryOp<T1, T2> for MemoryManager<T1, T2> {
    fn new(
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
            page: ContentMutex::new(page),
            memory: ContentMutex::new(T2::new(p)),
        }
    }

    fn kernel_page(&mut self, num : usize)->Option<*mut u8> {
        self.page.lock().alloc_kernel_page(num)
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

    fn free_kernel_memory(&mut self, addr : *mut u8) {
        self.memory.lock().free_kernel_memory(addr);
    }

    fn free_user_memory(&mut self, addr : *mut u8) {
        self.memory.lock().free_user_memory(addr);
    }

    fn print(&mut self) {
        self.page.lock().print();
        self.memory.lock().print();
    }
}