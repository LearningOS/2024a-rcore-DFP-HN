//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

// use super::manager::TASK_MANAGER;
use super:: __switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;
use crate::config::BIG_STRIDE;

/// 处理器管理结构：记录当前运行任务与空闲上下文，负责调度切换。
/// Processor management structure
pub struct Processor {
    /// 当前处理器上正在运行的任务；为 None 时表示空闲。
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    /// 每核的空闲任务上下文，用于回到空闲流触发调度与切换。
    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }

}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

/// 调度与进程执行的主循环。
/// - 持续调用 `fetch_task` 从就绪队列获取下一个要运行的任务；若无任务则记录告警并继续循环。
/// - 若当前有运行中的任务，按步长调度规则更新其 `stride` 值。
/// - 为即将运行的任务设置状态为 `Running`，必要时记录 `start_time`，并通过 `__switch`
///   从空闲上下文切换到该任务的上下文（`idle_task_cx_ptr` -> `next_task_cx_ptr`）。
/// - 该函数为无限循环，除非系统异常退出，否则不会返回。
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            if let Some(idle_task) = &processor.current {
                let mut idle_task = idle_task.inner_exclusive_access();
                idle_task.stride = (idle_task.stride + BIG_STRIDE as usize / idle_task.priority as usize) % (BIG_STRIDE + 1);
            }
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            if task_inner.start_time == 0 {
                task_inner.start_time = get_time_ms();
            }
            
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

/// 切回空闲调度上下文并触发新一轮调度。
///
/// - `switched_task_cx_ptr`：当前任务的 `TaskContext` 指针，切出前将寄存器保存到这里；
/// - 从 `PROCESSOR` 取出每核的 `idle_task_cx_ptr`，调用 `__switch(switched, idle)`，
///   将控制流“返回”到空闲调度循环，由调度器选择下一个任务并再度 `__switch` 进入；
/// - 本函数不做选择逻辑，仅负责上下文切换到空闲流。
///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }

}
