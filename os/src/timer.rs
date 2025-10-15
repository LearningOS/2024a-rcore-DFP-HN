//! RISC-V 计时器相关功能
//!
//! 提供：读取时间、毫秒/微秒换算、设置下一次时钟中断。

use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;
/// 每秒触发的时钟中断次数（调度节拍）
const TICKS_PER_SEC: usize = 100;
/// 每秒的毫秒数
const MSEC_PER_SEC: usize = 1000;
/// 每秒的微秒数
#[allow(dead_code)]
const MICRO_PER_SEC: usize = 1_000_000;

/// 获取当前时间（原始时钟 tick）
///
/// 返回：`time` CSR 的当前值
pub fn get_time() -> usize {
    time::read()
}

/// 获取当前时间（毫秒）
///
/// 返回：`time` 折算成毫秒（基于 `CLOCK_FREQ`）
#[allow(dead_code)]
pub fn get_time_ms() -> usize {
    time::read() * MSEC_PER_SEC / CLOCK_FREQ
}

/// 获取当前时间（微秒）
///
/// 返回：`time` 折算成微秒（基于 `CLOCK_FREQ`）
#[allow(dead_code)]
pub fn get_time_us() -> usize {
    time::read() * MICRO_PER_SEC / CLOCK_FREQ
}

/// 设置下一次时钟中断触发时刻
///
/// 策略：当前 tick 基础上加上 `CLOCK_FREQ / TICKS_PER_SEC`
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
