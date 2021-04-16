//! # 内存分配器
//! 将内存按照 2 的幂次对齐后进行分配
//! 分为内核、用户两种内存
//! 
//! 2021年1月25日 zg

pub struct Heap<T:PageOp> {
    page_manager : T,
    user_allocator : Option<*mut MemoryPool>,
    kernel_allocator : Option<*mut MemoryPool>,
    mutex : Mutex,
}

impl<T:PageOp> Heap<T> {
    fn alloc(&mut self, size : usize, is_kernel : bool)->Option<*mut u8> {
        let size = align(size);
        let node = self.find_first_contain(size, is_kernel);
        let rt;
        if let Some(node) = node {
            rt = node;
        }
        // 没有足够空间，申请新的
        else {
            rt = self.create_pool(size, is_kernel).unwrap();
        }
        let rt = unsafe{&mut *(rt)};
        if let Some(idx) = rt.bitmap.alloc_bitmap() {
            let addr = (idx * rt.size + rt.physic_base as usize) as *mut u8;
            self.clear(addr, size);
            Some(addr)
        }
        else {
            None
        }
    }

    fn create_pool(&mut self, size : usize, is_kernel : bool)->Option<*mut MemoryPool> {
        let num_alloc = self.decide_page_num(size);
        let bit_addr;
        let struct_addr;
        let free_cnt;
        let total_size = num_alloc * self.page_manager.page_size();
        let struct_size = (total_size / size + 7) / 8 + size_of::<MemoryPool>();
        let phy_addr ;
        if is_kernel {
            phy_addr = self.page_manager.alloc_kernel_page(num_alloc).unwrap();
        }
        else{
            phy_addr = self.page_manager.alloc_user_page(num_alloc).unwrap();
        }
        // 块的粒度较大时另外存放结构体
        if size > struct_size * 2 {
            if is_kernel {
                struct_addr = self.alloc(struct_size, true).unwrap()
            }
            else {
                struct_addr = self.alloc(struct_size, false).unwrap()
            }
            free_cnt = total_size / size;
        }
        // 如果较小，则直接放置在申请的页表内
        else {
            struct_addr = phy_addr;
            free_cnt = (total_size - struct_size) / size;
        }
        bit_addr = struct_addr as usize + size_of::<MemoryPool>();
        let t = struct_addr as *mut MemoryPool;
        unsafe {
            (*t).init(phy_addr as *mut u8,total_size,
                size, bit_addr as *mut u8, free_cnt);
            self.append(t, is_kernel);
        }
        Some(t)
    }

    fn free(&mut self, addr : *mut u8, is_kernel : bool) {
        let mut head;
        if is_kernel {head = self.kernel_allocator;}
        else {head = self.user_allocator;}

        let node;
        unsafe {
            while !(*head.unwrap()).is_contain(addr){
                head = (*head.unwrap()).next;
            }
            (*head.unwrap()).free_bitmap(addr);
            node = &mut *head.unwrap();
        }
        // 如果同大小空内存池太多，释放掉此内存池
        if node.bitmap.use_cnt == 0 {
            let size = node.size;
            let free_cnt = self.get_free_block_num(size, is_kernel);
            let use_cnt = self.get_used_block_num(size, is_kernel);
            if free_cnt <= 1 || free_cnt * 2 <= use_cnt { return; }

            // 如果块结构体在自己管理的页表内
            if node.is_inside() {
                self.page_manager.free_page(head.unwrap() as *mut u8);
            }
            else {
                self.page_manager.free_page(node.physic_base);
                self.free(head.unwrap() as *mut u8, is_kernel);
            }
            self.remove_pool(head.unwrap(), is_kernel);
        }
    }

    fn find_first_contain(&self, size : usize, is_kernel : bool)->Option<*mut MemoryPool> {
        let mut head;
        if is_kernel { head = self.kernel_allocator; }
        else { head = self.user_allocator; }

        while head.is_some() && !unsafe{(*head.unwrap()).can_contain(size)} {
            head = unsafe{(*head.unwrap()).next};
        }
        head
    }

    fn decide_page_num(&self, size : usize) -> usize{
        let page_size = self.page_manager.page_size();
        let too_big = MEMORY_TOO_BIG;
        if size > too_big {
            (size + page_size - 1) / page_size
        }
        else {
            (size * 4 + page_size - 1) / page_size
        }
    }

    fn append(&mut self, pool : *mut MemoryPool, is_kernel : bool) {
        let mut head;
        if is_kernel { head = self.kernel_allocator; }
        else { head = self.user_allocator; }
        if head.is_none() {
            if is_kernel {
                self.kernel_allocator = Some(pool);
            }
            else {
                self.user_allocator = Some(pool);
            }
            return;
        }

        let size = unsafe {(*pool).size};

        unsafe {
            let mut next = (*head.unwrap()).next;
            while next.is_some() && (*next.unwrap()).size < size {
                head = next;
                next = (*head.unwrap()).next;
            }
            if next.is_some() {
                (*pool).next = next;
            }
            (*head.unwrap()).next = Some(pool);
        }
    }

    fn get_free_block_num(&self, size : usize, is_kernel : bool)->usize {
        let mut head;
        if is_kernel { head = self.kernel_allocator; }
        else { head = self.user_allocator; }

        let mut cnt = 0;

        unsafe{
            while head.is_some() && (*head.unwrap()).size < size {
                head = (*head.unwrap()).next;
            }
            while head.is_some() && (*head.unwrap()).size == size {
                if (*head.unwrap()).bitmap.use_cnt == 0 {
                    cnt += 1;
                }
                head = (*head.unwrap()).next;
            }
        }
        cnt
    }

    fn get_used_block_num(&self, size : usize, is_kernel : bool)->usize {
        let mut head;
        if is_kernel { head = self.kernel_allocator; }
        else { head = self.user_allocator; }

        let mut cnt = 0;
    
        unsafe{
            while head.is_some() && (*head.unwrap()).size < size {
                head = (*head.unwrap()).next;
            }
            while head.is_some() && (*head.unwrap()).size == size {
                if (*head.unwrap()).bitmap.use_cnt != 0 {
                    cnt += 1;
                }
                head = (*head.unwrap()).next;
            }
        }
        cnt
    }

    fn remove_pool(&mut self, node : *mut MemoryPool, is_kernel : bool) {
        let mut head;
        if is_kernel { head = self.kernel_allocator; }
        else { head = self.user_allocator; }

        unsafe {
            while (*head.unwrap()).next.unwrap() != node {
                head = (*head.unwrap()).next;
            }

            (*head.unwrap()).next = (*(*head.unwrap()).next.unwrap()).next;
        }
    }

    fn clear(&mut self, addr : *mut u8, size : usize) {
        unsafe {
            addr.write_bytes(0, size);
        }
    }
}

impl<T:PageOp> HeapOp<T> for Heap<T> {
    fn new<'a>(page : T)->Self {
        Self {
            page_manager : page,
            kernel_allocator : None,
            user_allocator : None,
            mutex : Mutex::new(),
        }
    }

    fn alloc_kernel_memory(&mut self, size : usize)->Option<*mut u8> {
        self.mutex.lock();
        let rt = self.alloc(size, true);
        self.mutex.unlock();
        rt
    }

    fn alloc_user_memory(&mut self, size : usize)->Option<*mut u8> {
        self.mutex.lock();
        let rt = self.alloc(size, false);
        self.mutex.unlock();
        rt
    }

    fn free_kernel_memory(&mut self, addr : *mut u8) {
        self.mutex.lock();
        self.free(addr, true);
        self.mutex.unlock();
    }

    fn free_user_memory(&mut self, addr : *mut u8) {
        self.mutex.lock();
        self.free(addr, false);
        self.mutex.unlock();
    }

    fn print(&self) {
        let mut head = self.kernel_allocator;

        unsafe {
            while head.is_some() {
                let _t = &(*head.unwrap());
                head = (*head.unwrap()).next;
            }
        }
    }
}

pub struct MemoryPool {
    physic_base : *mut u8,
    size : usize,
    next : Option<*mut MemoryPool>,
    bitmap : Bitmap,
}

/// 将某个数向上取 2^n
fn align(x : usize) -> usize{
    let mut rt = 2;
    while rt < x {
        rt *= 2;
    }
    rt
}

/// ## 私有辅助方法
impl MemoryPool {
    pub fn can_contain(&mut self, size : usize)->bool {
        self.bitmap.free_cnt > 0 && self.size >= size
    }
    /// ### 初始化变量
    fn init(&mut self, addr : *mut u8, total_size : usize, sz : usize,
        bit_addr : *mut u8, free_cnt : usize) {
        let size = align(sz);
        self.physic_base = addr;
        let total_cnt = total_size / size;
        self.bitmap.init(bit_addr, total_cnt, free_cnt);
        self.size = size;
        // self.bitlen = self.total_cnt / 8;
        self.next = None;
    }

    fn is_inside(&self)->bool {
        self.size < MEMORY_SIZE_INSIDE
    }
    /// ### 根据地址找到对应的元素然后释放
    fn free_bitmap(&mut self, addr : *mut u8){
        let st = self.physic_base as usize;
        let idx = (addr as usize - st) / self.size;
        self.bitmap.free(idx);
    }
    /// 元素是否包含此地址
    fn is_contain(&self, addr : *mut u8) -> bool {
        let adr = addr as usize;
        let st = self.physic_base as usize;
        let ed = st + self.bitmap.total_cnt * self.size;
        adr >= st && adr < ed
    }
}

const MEMORY_TOO_BIG : usize = 4096;
const MEMORY_SIZE_INSIDE : usize = 256;


use core::{mem::size_of};

use tisu_sync::Mutex;

use crate::{bitmap::Bitmap, require::{HeapOp, PageOp}};
