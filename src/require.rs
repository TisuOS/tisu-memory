//! # 接口要求
//! 
//! 2021年4月14日 zg


/// ## 页面管理
/// 页面管理将内存按照 page_size 大小分页，对外提供申请、释放功能
pub trait PageOp {
    fn clone(&self)->Self;
    fn new(kmem_start : usize, umem_start : usize,
        total_mem : usize, page_size : usize)->Self;
    fn alloc_kernel_page(&mut self, num : usize)->Option<*mut u8>;
    fn alloc_user_page(&mut self, num : usize)->Option<*mut u8>;
    fn free_page(&mut self, addr : *mut u8);
    fn page_size(&self)->usize;
    fn print(&self);
}

/// ## 堆内存管理
/// 基于页面管理提供任意大小的内存分配功能
pub trait HeapOp<T:PageOp> {
    fn new(page : T)->Self;
    fn alloc_kernel_memory(&mut self, size : usize)->Option<*mut u8>;
    fn alloc_user_memory(&mut self, size : usize)->Option<*mut u8>;
    fn free_kernel_memory(&mut self, addr : *mut u8);
    fn free_user_memory(&mut self, addr : *mut u8);
    fn print(&self);
}

/// ## 内存管理接口
/// 统御堆内存、页面管理，作为对外提供功能的接口
pub trait MemoryOp<T1:PageOp, T2:HeapOp<T1>> {
    fn new(
        heap_start : usize,
        kernel_page_num : usize,
        page_size : usize,
        memory_end : usize
    )->Self;

    fn kernel_page(&mut self, num : usize)->Option<*mut u8>;

    fn user_page(&mut self, num : usize)->Option<*mut u8>;

    fn free_page(&mut self, addr : *mut u8);

    fn alloc_memory(&mut self, size : usize, is_kernel : bool)->Option<*mut u8>;

    fn free_kernel_memory(&mut self, addr : *mut u8);

    fn free_user_memory(&mut self, addr : *mut u8);

    fn print(&mut self);
}

pub trait AutoMemory<T1:Copy> : Drop {
    fn new(size : usize)->Self;

    fn get(&self, idx : usize)->Option<T1>;

    fn set(&self, idx : usize, val : T1, len : usize);

    /// ## 拷贝
    /// 长度的单位以 other 为准
    fn copy_to<T2:Copy>(&self, st1 : usize, other : &impl AutoMemory<T2>, st2 : usize, len : usize);

    /// ## 拷贝
    /// 长度的单位以 other 为准
    fn copy_from<T2:Copy>(&self, st1 : usize, other : &impl AutoMemory<T2>, st2 : usize, len : usize);

    fn get_addr(&self)->usize;

    fn size(&self)->usize;
}