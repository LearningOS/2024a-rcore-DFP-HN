//! Process management syscalls
// use riscv::addr::VirtAddr;

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{translate_virt_phy, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER
    },
    timer::{get_time_ms, get_time_us},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let time_us = get_time_us();
    let token = current_user_token();
    let virt_sec = unsafe { VirtAddr(&(*_ts).sec as *const usize as usize)};
    if let Some(phy_sec) = translate_virt_phy(virt_sec, token) {
        let phy_sec = (phy_sec.0 << 12 | virt_sec.page_offset()) as *mut usize;
        unsafe {
            *phy_sec = time_us / 1_000_000;
        }
    }
    else {
        return -1;
    }
    let virt_usec = unsafe { VirtAddr(&(*_ts).usec as *const usize as usize)};
    if let Some(phy_usec) = translate_virt_phy(virt_usec, token) {
        let phy_usec = (phy_usec.0 << 12 | virt_usec.page_offset()) as *mut usize;
        unsafe {
            *phy_usec = time_us % 1_000_000;
        }
    }
    else {
        return -1;
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let token = current_user_token();
    let virt_status = unsafe { VirtAddr(&(*_ti).status as *const TaskStatus as usize)};
    let time_ms = get_time_ms();
    if let Some(phy_status) = translate_virt_phy(virt_status, token) {
        let phy_status = (phy_status.0 << 12 | virt_status.page_offset()) as *mut TaskStatus;
        unsafe {
            *phy_status = TaskStatus::Running;
        }
    }
    else {
        return -1;
    }
    for i in 0..MAX_SYSCALL_NUM {
        let virt_syscall_times = unsafe {VirtAddr(&(*_ti).syscall_times[i] as *const u32 as usize)};
        if let Some(phy_syscall_times) = translate_virt_phy(virt_syscall_times, token) {
            let phy_syscall_times = (phy_syscall_times.0 << 12 | virt_syscall_times.page_offset()) as *mut u32;
            unsafe {
                *phy_syscall_times = TASK_MANAGER.get_syscall_times(i);
            }
        }
        else {
            return -1;
        }
    }
    let virt_time = unsafe { VirtAddr(&(*_ti).time as *const usize as usize)};
    if let Some(phy_time) = translate_virt_phy(virt_time, token) {
        let phy_time = (phy_time.0 << 12 | virt_time.page_offset()) as *mut usize;
        unsafe {
            *phy_time = time_ms - TASK_MANAGER.get_start_time();
        }
    }
    else {
        return -1;
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if (_start & (PAGE_SIZE - 1) != 0) || (_port & !0x7 != 0) || (_port & 0x7 == 0) {
        println!("sys_mmap error occur!");
        return -1;
    }
    TASK_MANAGER.mmap(VirtAddr(_start), VirtAddr(_start + _len), _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    let vpn_start = VirtAddr(_start).floor();
    let vpn_end = VirtAddr(_start + _len).ceil();
    TASK_MANAGER.munmap(vpn_start.into(), vpn_end.into())

}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
