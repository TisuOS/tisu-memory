//! # 内存管理器
//! MemoryOp 为内存使用接口
//! tisu-memory 提供默认实现
//! 其中 PageOp、HeapOp 为默认实现 MemoryManager 的接口要求
//! 
//! 2021年4月14日 zg

#![no_std]

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
