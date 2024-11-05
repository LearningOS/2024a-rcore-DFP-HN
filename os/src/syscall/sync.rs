use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use alloc::vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        for allocate in process_inner.allocate_mutex.iter_mut() {
            allocate[id] = 0;
        }
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        for allocate in process_inner.allocate_mutex.iter_mut() {
            allocate.push(0);
        }
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    process_inner.need_mutex[tid] = mutex_id as isize;
    if process_inner.deadlock_detect {
        let task_len = process_inner.tasks.len();
        let mutex_len = process_inner.mutex_list.len();
        let len = task_len + mutex_len;
        let mut in_degree = vec![0; len];
        for i in 0..task_len {
            if process_inner.need_mutex[i] != -1 {
                in_degree[task_len + process_inner.need_mutex[i] as usize] += 1;
            }
        }
        for i in 0..task_len {
            for j in 0..mutex_len {
                if process_inner.allocate_mutex[i][j] != 0 {
                    in_degree[i] += 1;
                }
            }
        }
        let mut que: VecDeque<usize> = VecDeque::new();
        for i in 0..len {
            debug!("indegree[{}] = {}", i, in_degree[i]);
            if in_degree[i] == 0 {
                que.push_back(i);
            }
        }
        let mut cnt = 0;
        while !que.is_empty() {
            cnt += 1;
            let u = que.pop_front().unwrap();
            if u < task_len {
                for i in 0..mutex_len {
                    if process_inner.need_mutex[i] != -1 {
                        in_degree[task_len + i] -= 1;
                        if in_degree[task_len + i] == 0 {
                            que.push_back(task_len + i);
                        }
                    }
                }
            }
            else {
                for i in 0..task_len {
                    if process_inner.allocate_mutex[i][u - task_len] != 0 {
                        in_degree[i] -= 1;
                        if in_degree[i] == 0 {
                            que.push_back(i);
                        }
                    }
                }
            }
        }
        if cnt != len {
            process_inner.need_mutex[mutex_id] = -1;
            return -0xDEAD;
        }
    }
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.lock();
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.allocate_mutex[tid][mutex_id] = 1;
    process_inner.need_mutex[tid] = -1;
    drop(process_inner);
    drop(process);
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    drop(process_inner);
    drop(process);
    mutex.unlock();
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.allocate_mutex[tid][mutex_id] = 0;
    drop(process_inner);
    drop(process);
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        process_inner.available_semaphore[id] = res_count;
        for allocate in process_inner.allocate_semaphore.iter_mut() {
            allocate[id] = 0;
        }
        for need in process_inner.need_semaphore.iter_mut() {
            need[id] = 0;
        }
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.available_semaphore.push(res_count);
        for allocate in process_inner.allocate_semaphore.iter_mut() {
            allocate.push(0);
        }
        for need in process_inner.need_semaphore.iter_mut() {
            need.push(0);
        }
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.up();
    let mut process_inner = process.inner_exclusive_access();
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    process_inner.allocate_semaphore[tid][sem_id] -= 1;
    process_inner.available_semaphore[sem_id] += 1;
    drop(process_inner);
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    process_inner.need_semaphore[tid][sem_id] += 1;
    if process_inner.deadlock_detect {
        let task_len = process_inner.tasks.len();
        let semaphore_len = process_inner.semaphore_list.len();
        let mut finish = vec![false;task_len];
        let mut work = process_inner.available_semaphore.clone();
        let mut cnt = 0;
        let mut i = 0;
        while i < task_len {
            if finish[i] == false {
                let mut flag = true;
                for j in 0..semaphore_len {
                    if process_inner.need_semaphore[i][j] > 0 && work[j] < process_inner.need_semaphore[i][j] {
                        flag = false;
                        break;
                    }
                }
                if flag {
                    cnt += 1;
                    finish[i] = true;
                    for j in 0..semaphore_len {
                        work[j] += process_inner.allocate_semaphore[i][j];
                    }
                    i = 0;
                    continue;
                }
            }
            i += 1;
        }
        if cnt < task_len {
            process_inner.need_semaphore[tid][sem_id] -= 1;
            return -0xDEAD;
        }
    }
    drop(process_inner);
    sem.down();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.need_semaphore[tid][sem_id] -= 1;
    process_inner.allocate_semaphore[tid][sem_id] += 1;
    process_inner.available_semaphore[sem_id] -= 1;
    drop(process_inner);
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    if _enabled > 1 {
        return -1;
    }
    else {
        let process = current_process();
        let mut process_inner = process.inner_exclusive_access();
        if _enabled == 1 {
            process_inner.deadlock_detect = true;
        }
        else {
            process_inner.deadlock_detect = false;
        }
    }
    -1
}
