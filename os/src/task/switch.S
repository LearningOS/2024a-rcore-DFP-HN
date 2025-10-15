.altmacro # 启用 GAS 替代宏，便于参数化保存/恢复 s 寄存器
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)    
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)    
.endm
    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # 约定：
    # - a0 = current_task_cx_ptr（当前任务的 TaskContext 指针）
    # - a1 = next_task_cx_ptr（下一个任务的 TaskContext 指针）
    # - TaskContext 布局（单位字节）：
    #   [0]  = ra（返回地址寄存器，x1）
    #   [8]  = sp（栈指针，x2）
    #   [16] 起每 8 字节依次为 s0~s11（x8..x9, x18..x27），即 (n+2)*8 偏移
    # save kernel stack of current task
    sd sp, 8(a0)             # 保存当前任务的内核栈指针 sp 到 current_task_cx_ptr->sp
    # save ra & s0~s11 of current execution
    sd ra, 0(a0)             # 保存返回地址 ra 到 current_task_cx_ptr->ra
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # restore ra & s0~s11 of next execution
    ld ra, 0(a1)             # 恢复下一个任务保存的 ra
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # restore kernel stack of next task
    ld sp, 8(a1)             # 恢复下一个任务的内核栈指针 sp
    ret                      # 跳转到 ra 指向的位置：
                             # - 若任务首次运行，ra 预置为 trap_return；
                             # - 否则返回到该任务上次被切出时的内核位置。

