//! Implementation of [`TrapContext`]
use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
#[derive(Debug)]
/// Trap 上下文：保存用户通用寄存器、sstatus、sepc 及进入内核所需元信息。
/// 布局按 C 兼容顺序固定（repr(C)），汇编 `trap.S` 依赖固定偏移读写：
/// x[0..31]、[32]=sstatus、[33]=sepc、[34]=kernel_satp、[35]=kernel_sp、[36]=trap_handler。
/// trap context structure containing sstatus, sepc and registers
pub struct TrapContext {
    /// 用户通用寄存器镜像 x0..x31（例如：x2=sp，x10..x12=a0..a2 参数，x17=a7 为 syscall 号）。
    /// General-Purpose Register x0-31
    pub x: [usize; 32],
    /// 监督态状态寄存器 sstatus：保存返回前的特权/中断位（如 SPP/SIE/SPIE），在 `__restore` 中恢复。
    /// Supervisor Status Register
    pub sstatus: Sstatus,
    /// 监督态异常 PC sepc：返回到用户态时的程序计数器；系统调用场景内核会将其自增 4 跳过 ecall。
    /// Supervisor Exception Program Counter
    pub sepc: usize,
    /// 内核地址空间 token（satp）：陷入后在 `__alltraps` 中切换到该页表以运行内核代码。
    /// Token of kernel address space
    pub kernel_satp: usize,
    /// 当前应用的内核栈指针（kernel stack top）：陷入后切换到此栈作为内核栈。
    /// Kernel stack pointer of the current application
    pub kernel_sp: usize,
    /// 内核陷入处理函数入口的虚拟地址：`__alltraps` 读取该值并跳转到内核 `trap_handler`。
    /// Virtual address of trap handler entry point in kernel
    pub trap_handler: usize,
}

impl TrapContext {
    /// put the sp(stack pointer) into x\[2\] field of TrapContext
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    /// init the trap context of an application
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        // set CPU privilege to User after trapping back
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,  // entry point of app
            kernel_satp,  // addr of page table
            kernel_sp,    // kernel stack
            trap_handler, // addr of trap_handler function
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}
