//! Process management syscalls
// use alloc::sync::Arc;

use crate::{
    config::{BIG_STRIDE, MAX_SYSCALL_NUM, PAGE_SIZE},
    loader::get_app_data_by_name,
    mm::{translate_virt_phy, translated_refmut, translated_str, VirtAddr},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus,
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
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        // assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
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
        0
    }
    else {
        return -1;
    }
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!(
        "kernel:pid[{}] sys_task_info NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
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
                let current_task = current_task().unwrap();
                *phy_syscall_times = current_task.get_syscall_times(i);
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
            let current_task = current_task().unwrap();
            *phy_time = time_ms - current_task.get_start_time();
        }
        0
    }
    else {
        return -1;
    }
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if (_start & (PAGE_SIZE - 1) != 0) || (_port & !0x7 != 0) || (_port & 0x7 == 0) {
        return -1;
    }
    let current_task = current_task().unwrap();
    current_task.mmap(VirtAddr(_start), VirtAddr(_start + _len), _port)
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    let vpn_start = VirtAddr(_start).floor();
    let vpn_end = VirtAddr(_start + _len).ceil();
    let current_task = current_task().unwrap();
    current_task.munmap(vpn_start.into(), vpn_end.into())
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let path = translated_str(token, _path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        let new_task = task.spawn(data);
        let new_pid = new_task.pid.0;
        add_task(new_task);
        new_pid as isize
    } else {
        -1
    }
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if _prio < 2 || _prio as usize > BIG_STRIDE {
        return -1;
    }
    let current_task = current_task().unwrap();
    let mut inner = current_task.inner_exclusive_access();
    inner.priority = _prio;
    _prio
}
