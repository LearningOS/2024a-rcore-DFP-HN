# Lab1报告
## 实现的功能
+ 在TaskControlBlock中加入了start_time和syscall_times分别用来记录程序第一次被cpu调度的时间和该程序使用系统调用的次数。
+ 在run_first_task函数中记录第一个程序的start_time。
+ 在run_next_task函数中判断程序是否第一次被cpu调度，如果是，则记录该程序的start_time。
+ 在TaskManager中加入了update_syscall_times(&self, sid: usize)，sid是系统调用ID，update_syscall_times用来更新当前运行程序的系统调用次数，该函数在每次程序进行系统调用前调用。
+ 在TaskManager中加入了get_task_info(&self, time: &mut usize, syscall_times: &mut [u32; MAX_SYSCALL_NUM]),该函数用来获取task_info的time和syscall_time。该函数在sys_task_info函数中使用。

# 简答作业
1. RustSBI-QMEU Version 0.2.0-alpha.2
ch2b_bad_address.rs的错误信息：PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
ch2b_bad_instructions.rs的错误信息：IllegalInstruction in application, kernel killed it.
ch2b_bad_register.rs的错误信息：IllegalInstruction in application, kernel killed it.
2. 

    1. 刚进入 __restore 时，a0代表trap_handler的返回值。在系统调用和程序调度时会使用 __restore。
    2. 
    ```Rust
    ld t0, 32*8(sp) // 32*8(sp)处存储执行trap_handler前的sstatus值。
    ld t1, 33*8(sp) // 33*8(sp)处存储执行trap_handler前的spec值
    ld t2, 2*8(sp) // 2*8(sp)处存储用户栈的地址，将用户栈地址加载到t2中。
    csrw sstatus, t0 // 恢复trap_handler执行前的status值。
    csrw sepc, t1 // 恢复trap_handler执行前的spec值。
    csrw sscratch, t2 //将用户栈的地址加载到sscratch中。
    ```
3. x2是sp，不需要保存在栈中，x4是tp（线程指针），指向当前线程的上下文，主要用于多线程环境。每个线程都有自己的上下文，当线程切换时，操作系统会保存当前线程的所有寄存器状态（包括 tp）到线程控制块（TCB）中。在 RISC-V 的调用约定中，tp 作为一个特殊寄存器，不会在函数调用中被调用者保留。它的值可能在函数调用期间被覆盖，因此在正常情况下，调用者不需要保存 tp 的值到栈中。在上下文切换时，操作系统会负责保存和恢复 tp 的值，因此应用程序通常不需要管理这个寄存器。

4. csrrw sp, sscratch, sp，这条指令将会交换sscratch交换sp的值，指令执行前sscratch存储用户栈地址，sp存储内核栈地址，交换后sscratch存储内核栈地址，sp存储用户栈地址。
5. 状态切换发生在 sret 指令上。执行 sret 之后，处理器会从超级用户模式（supervisor mode）切换回用户模式（user mode）。这条指令用于返回到之前保存的用户上下文。执行 sret 时，处理器会读取 sstatus 中的特权级信息，如果 sstatus 中的特权级设置为用户模式，处理器将切换到用户模式。时，sepc 中的值会作为新的程序计数器（PC）值，从而指向用户态程序的下一条指令。
6. 这条指令将会交换sscratch交换sp的值，指令执行后，sscratch存储用户栈地址，sp存储内核栈地址。
7. 从用户态（U 态）进入超级用户态（S 态）的过程是通过 csrrw sp, sscratch, sp 这条指令发生的。

<br>

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    无

2. 此外，我也参考了以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    [rCore-Camp-Guide-2024A文档](https://learningos.cn/rCore-Camp-Guide-2024A/chapter3/5exercise.html)

    [ChatGPT](https://chatgpt.com/)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。