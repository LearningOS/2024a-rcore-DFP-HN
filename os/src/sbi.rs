//! SBI 调用封装（Supervisor Binary Interface）
//!
//! 提供定时器、控制台 I/O、关机等 M 态服务的薄封装。

#![allow(unused)]

use core::arch::asm;

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_SHUTDOWN: usize = 8;

/// 进行一次通用的 SBI 调用（通过 ecall 陷入 M 态固件）
///
/// 参数：
/// - `which`: 功能号（如定时器/控制台等）
/// - `arg0`: 第 1 个参数（传入 a0）
/// - `arg1`: 第 2 个参数（传入 a1）
/// - `arg2`: 第 3 个参数（传入 a2）
///
/// 返回：
/// - `usize`: 返回值（从 a0 读取）
///
/// 约定：将 a7(which)/a6(0)/a0..a2 设置后执行 ecall。
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x16") 0,
            in("x17") which,
        );
    }
    ret
}

/// 设置下一次定时器中断触发的时间戳（CPU 时钟计数）
///
/// 参数：
/// - `timer`: 触发时刻（时钟 tick）
pub fn set_timer(timer: usize) {
    sbi_call(SBI_SET_TIMER, timer, 0, 0);
}

/// 向控制台输出一个字符（通过 UART）
///
/// 参数：
/// - `c`: 字符的 ASCII 码
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// 从控制台读取一个字符（非阻塞）
///
/// 返回：
/// - 如果有输入，返回 ASCII 码；否则通常返回特定占位值（依固件实现）
pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}

/// 请求关机（由 M 态固件执行）。若未关机将触发 panic。
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}
