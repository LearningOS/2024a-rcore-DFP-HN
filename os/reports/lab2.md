# Lab2报告
## 实现功能
+ 在TaskControlBlock中加入了start_time和syscall_times分别用来记录程序第一次被cpu调度的时间和该程序使用系统调用的次数。
+ 在run_first_task函数中记录第一个程序的start_time。 
+ 在run_next_task函数中判断程序是否第一次被cpu调度，如果是，则记录该程序的start_time。
+ 在TaskManager中加入了update_syscall_times(&self, idx: usize)，idx是系统调用ID，update_syscall_times用来更新当前运行程序的系统调用次数，该函数在每次程序进行系统调用前调用。
+ 在TaskManager中加入了get_start_time(&self),该函数用来获取应用的start_time。该函数在sys_task_info函数中使用。
+ 在TaskManager中加入了get_syscall_times(&self, idx: usize),该函数用来获取应用的syscall_times[idx]。该函数在sys_task_info函数中使用。