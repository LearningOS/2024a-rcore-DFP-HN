# Lab1报告
## 实现的功能
+ 在TaskControlBlock中加入了start_time和syscall_times分别用来记录程序第一次被cpu调度的时间和该程序使用系统调用的次数。
+ 在run_first_task函数中记录第一个程序的start_time。
+ 在run_next_task函数中判断程序是否第一次被cpu调度，如果是，则记录该程序的start_time。
+ 在TaskManager中加入了update_syscall_times(&self, sid: usize)，sid是系统调用ID，update_syscall_times用来更新当前运行程序的系统调用次数，该函数在每次程序进行系统调用前调用。
+ 在TaskManager中加入了get_task_info(&self, time: &mut usize, syscall_times: &mut [u32; MAX_SYSCALL_NUM]),该函数用来获取task_info的time和syscall_time。该函数在sys_task_info函数中使用。

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

此外，我也参考了以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[rCore-Camp-Guide-2024A文档](https://learningos.cn/rCore-Camp-Guide-2024A/chapter3/5exercise.html)

[ChatGPT](https://chatgpt.com/)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。