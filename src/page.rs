//! # 内存页管理
//! 将内存分页进行管理，方便后期页表映射
//! 
//! 2021年1月25日 zg

use core::ptr::slice_from_raw_parts;

use crate::require::PageOp;


pub struct PageManager {
	kernel_page : &'static mut [Page],
	kernel_page_num : usize,
	kernel_start : usize,
	user_page : &'static mut [Page],
	user_page_num : usize,
	user_start : usize,
	total_num : usize,
	memory_end : usize,
	page_size : usize,
}


impl PageManager {
	fn clear(&self, addr : *mut u8, num : usize) {
		unsafe {
			let len = num * self.page_size;
			addr.write_bytes(0, len);
		}
	}

	fn init_page(&mut self) {
		let rev_num = (self.total_num + self.page_size - 1) / self.page_size;
		let ptr = &mut self.kernel_page;
		for i in 0..rev_num {
			ptr[i].take();
		}
		for i in rev_num..self.kernel_page_num {
			ptr[i].free();
		}
		let ptr = &mut self.user_page;
		for i in 0..self.user_page_num {
			ptr[i].free();
		}
	}
}

impl PageOp for PageManager {
	fn clone(&self) -> Self {
		let kernel_page;
		let user_page;
		unsafe {
			let t = self as *const Self as *mut Self;
			kernel_page = &mut *((*t).kernel_page.as_ref() as *const [Page] as *mut [Page]);
			user_page = &mut *((*t).user_page.as_ref() as *const [Page] as *mut [Page]);
		}
        Self {
            kernel_page : kernel_page,
            kernel_page_num: self.kernel_page_num,
            user_page : user_page,
			kernel_start : self.kernel_start,
            user_page_num: self.user_page_num,
            user_start: self.user_start,
            total_num: self.total_num,
            memory_end: self.memory_end,
            page_size: self.page_size,
		}
    }

    fn new(kmem_start : usize, umem_start : usize,
			total_mem : usize, page_size : usize)->Self {
		let kmem_start = (kmem_start + page_size - 1) / page_size * page_size;
		let umem_start = (umem_start + page_size - 1) / page_size * page_size;
		let total_num = (total_mem - kmem_start) / page_size;
		let kernel_page_num = (umem_start - kmem_start) / page_size;
		let kernel_page = slice_from_raw_parts(kmem_start as *mut Page,
										kernel_page_num) as *mut [Page];
		let kernel_page = unsafe{&mut *(kernel_page)};
		let user_page = slice_from_raw_parts(
			(kmem_start + kernel_page_num) as *mut Page,
			total_num - kernel_page_num) as *mut [Page];
		let user_page = unsafe{&mut *(user_page)};
		
		let mut rt = Self {
		    kernel_page : kernel_page,
		    kernel_page_num,
			kernel_start : kmem_start,
		    user_page : user_page,
		    user_page_num: total_num - kernel_page_num,
			user_start : umem_start,
		    total_num,
			memory_end : total_mem,
		    page_size,
		};
		rt.init_page();
		rt
    }

    fn alloc_kernel_page(&mut self, num : usize)->Option<*mut u8> {
		assert!(num > 0);
		let ptr = &mut self.kernel_page;
		let mut cnt = 0;
		for i in 0..self.kernel_page_num {
			if ptr[i].is_free() {
				cnt += 1;
			}
			else {
				cnt = 0;
			}
			if cnt >= num {
				for idx in i + 1 - cnt..=i {
					ptr[idx].take();
				}
				ptr[i].end();
				let addr = ((i + 1 - cnt) * self.page_size +
					self.kernel_start) as *mut u8;
				self.clear(addr, num);
				return Some(addr);
			}
		}
		None
    }

    fn alloc_user_page(&mut self, num : usize)->Option<*mut u8> {
		assert!(num > 0);
		let ptr = &mut self.user_page;
		let mut cnt = 0;
		for i in 0..self.user_page_num {
			if ptr[i].is_free() {
				cnt += 1;
			}
			else {
				cnt = 0;
			}
			if cnt >= num {
				for idx in i + 1 - cnt..=i {
					ptr[idx].take();
				}
				ptr[i].end();
				let addr = ((i + 1 - cnt) * self.page_size + self.user_start) as *mut u8;
				self.clear(addr, num);
				return Some(addr);
			}
		}
		panic!("cnt {} num {} user num {}", cnt, num, self.user_page_num);
    }

    fn free_page(&mut self, addr : *mut u8) {
		if addr as usize >= self.kernel_start && (addr as usize) < self.user_start {
			let mut idx = (addr as usize - self.kernel_start) / self.page_size;
			let ptr = &mut self.kernel_page;
			while !ptr[idx].is_end() {
				ptr[idx].free();
				idx += 1;
			}
			ptr[idx].free();
		}
		else if addr as usize >= self.user_start && (addr as usize) < self.memory_end {
			let mut idx = (addr as usize - self.user_start) / self.page_size;
			let ptr = &mut self.user_page;
			while !ptr[idx].is_end() {
				ptr[idx].free();
				idx += 1;
			}
			ptr[idx].free();
		}
		else {
			panic!("page out of range: {:x}, kernel {:x}, user {:x} end {:x}",
				addr as usize, self.kernel_start, self.user_start, self.memory_end);
		}
    }

    fn page_size(&self)->usize {
		self.page_size
    }

    fn print(&self) {
		let mut _cnt = 0;
		let ptr = &self.kernel_page;
		for i in 0..self.kernel_page_num {
			if !ptr[i].is_free() {
				_cnt += 1;
			}
			else {
				break;
			}
		}
    }
}

#[derive(Copy, Clone)]
pub struct Page{
	pub flag : u8
}

impl Page {
	pub fn take(&mut self){
		self.flag = PageBit::Taken.val();
	}
	pub fn end(&mut self){
		self.flag |= PageBit::End.val();
	}
	pub fn is_free(&self)->bool{
		self.flag == 0
	}
	pub fn free(&mut self) {
		self.flag = 0;
	}
	pub fn is_end(&self)->bool {
		self.flag & PageBit::End.val() != 0
	}
}

#[derive(Copy, Clone)]
pub enum PageBit{
	Taken = 1 << 0,
	End = 1 << 1,
}

impl PageBit {
	pub const fn val(self) -> u8{
		self as u8
	}
}

