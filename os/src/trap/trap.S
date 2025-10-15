.altmacro                         # 启用 GAS 替代宏，便于宏参数与数字拼接
.macro SAVE_GP n
    sd x\n, \n*8(sp)          
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)           
.endm
    .section .text.trampoline    # TRAMPOLINE 代码节：各进程虚拟地址一致
    .globl __alltraps            # 导出陷入统一入口符号
    .globl __restore             # 导出返回用户态入口符号
    .align 2                     # 2^2=4 字节对齐
__alltraps:                      # 用户态陷入后的第一站（在 TRAMPOLINE 页）
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)               # 保存 x1=ra（返回地址）
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)               # 保存 x3=gp（全局指针）
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5                    # 从 x5（t0）开始
    .rept 27                     # 循环保存 x5..x31（t0..t6,s0..s11,a0..a7,...）
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus             # 读取 sstatus（特权与中断位）
    csrr t1, sepc                # 读取 sepc（返回到用户态的 PC）
    sd t0, 32*8(sp)              # 保存 sstatus -> TrapContext[32]
    sd t1, 33*8(sp)              # 保存 sepc    -> TrapContext[33]
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch            # 取回用户态 sp（保存在 sscratch）
    sd t2, 2*8(sp)               # 保存用户态 sp -> TrapContext[2]
    # load kernel_satp into t0
    ld t0, 34*8(sp)              # 载入内核页表 token（satp）-> t0
    # load trap_handler into t1
    ld t1, 36*8(sp)              # 载入内核 trap_handler 虚拟地址 -> t1
    # move to kernel_sp
    ld sp, 35*8(sp)              # 切换到内核栈指针（kernel_sp）
    # switch to kernel space
    csrw satp, t0                # 切换到内核页表
    sfence.vma                   # 刷新 TLB
    # jump to trap_handler
    jr t1                        # 跳转进入内核 trap 处理函数

__restore:                       # 从内核返回用户态的入口（在 TRAMPOLINE 页）
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1                # 切换到用户页表（satp = a1）
    sfence.vma                   # 刷新 TLB
    csrw sscratch, a0            # sscratch = TrapContext 指针（供下一次陷入使用）
    mv sp, a0                    # sp 指向 TrapContext（在用户空间）
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)              # 取回保存的 sstatus
    ld t1, 33*8(sp)              # 取回保存的 sepc
    csrw sstatus, t0             # 恢复 sstatus
    csrw sepc, t1                # 恢复 sepc（用户态返回 PC）
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)               # 恢复 x1=ra
    ld x3, 3*8(sp)               # 恢复 x3=gp
    .set n, 5                    # 恢复 x5..x31
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)               # 切换回用户态的 sp
    sret                         # 依据 sstatus/sepc 返回到用户态继续执行
