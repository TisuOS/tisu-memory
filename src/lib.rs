//! # 内存管理器
//! 提供页面、堆内存的管理功能
//! 提供页面大小；内核、用户内存范围即可使用
//! 同时提供一个自动释放的内存块 Block
//! 
//! 2021年4月14日 zg

#![no_std]
#![feature(
    panic_info_message,
)]

mod lang_items;
mod require;
mod page;
mod heap;
mod bitmap;
mod config;
mod manager;

pub use require::{
    PageOp,
    HeapOp,
    MemoryOp,
    AutoMemory,
};

pub use heap::Heap;
pub use page::PageManager;
pub use manager::MemoryManager;
