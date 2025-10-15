//!Implementation of [`TaskManager`]


use super::TaskControlBlock;
use crate::config::BIG_STRIDE;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// 从就绪队列中选择并取出下一个要运行的任务（Stride 调度）。
    ///
    /// 遍历 `ready_queue`，比较各任务的 `stride` 值，在 \[0, BIG_STRIDE] 的环上
    /// 选择“最小 stride”的任务：
    /// - 若 `task_stride > current && task_stride - current > BIG_STRIDE / 2`，
    ///   视为跨越取模边界，认为 `task_stride` 更小；
    /// - 若 `task_stride < current && current - task_stride <= BIG_STRIDE / 2`，
    ///   认为 `task_stride` 更小；
    ///
    /// 该函数时间复杂度为 O(n)，若队列为空返回 `None`；否则将选中的任务从就绪队列
    /// 中移除并返回。注意：本函数不修改任务的 `stride`，其更新在调度点完成。
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut current = 0;
        let mut idx :isize = -1;
        let mut i = 0;
        for task in self.ready_queue.iter() {
            let task_stride = task.inner_exclusive_access().stride;
            if idx == -1 {
                current = task_stride;
                idx = i;
            }
            else if task_stride > current && task_stride - current > BIG_STRIDE / 2 {
                current = task_stride;
                idx = i;
            }
            else if task_stride < current && current - task_stride <= BIG_STRIDE / 2 {
                current = task_stride;
                idx = i;
            }
            i += 1;
        }
        if idx == -1 {
            None
        }
        else {
            self.ready_queue.remove(idx as usize)
        }
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
