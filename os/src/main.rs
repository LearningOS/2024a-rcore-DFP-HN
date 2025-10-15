//! 内核主模块与入口
//!
//! 子模块概览：
//! - [`trap`]: 用户态到内核态的陷入与返回
//! - [`task`]: 任务管理与调度
//! - [`syscall`]: 系统调用实现
//! - [`mm`]: 基于 SV39 的内存映射与管理
//! - [`sync`]: 基于静态数据结构的安全封装
//!
//! 启动流程：从 `entry.asm` 进入，随后调用 [`rust_main()`] 初始化各子系统，
//! 最终进入 [`task::run_tasks()`] 开始调度并首次进入用户态。

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

extern crate alloc;

#[macro_use]
mod console;
pub mod config;
pub mod lang_items;
mod loader;
pub mod logging;
pub mod mm;
pub mod sbi;
pub mod sync;
pub mod syscall;
pub mod task;
pub mod timer;
pub mod trap;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));
/// 清零 `.bss` 段（为未初始化的全局/静态变量提供 0 初始值）
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// 打印内核日志与关键段边界信息（便于启动期诊断）
fn kernel_log_info() {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    logging::init();
    println!("[kernel] Hello, world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
}

#[no_mangle]
/// Rust 侧的内核入口（不返回）
///
/// 步骤：
/// 1) 清零 bss 并打印段信息
/// 2) 初始化内存管理并自检
/// 3) 创建初始用户进程（initproc）
/// 4) 初始化陷入/中断与时钟中断
/// 5) 列出可加载的用户程序
/// 6) 进入调度主循环
pub fn rust_main() -> ! {
    clear_bss();
    kernel_log_info();
    mm::init();
    mm::remap_test();
    task::add_initproc();
    println!("after initproc!");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    loader::list_apps();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
