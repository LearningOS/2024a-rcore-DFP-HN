//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].

mod context;

use crate::config::{TRAMPOLINE, TRAP_CONTEXT_BASE};
use crate::syscall::syscall;
use crate::task::{
    current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, current_task,
};
use crate::timer::set_next_trigger;
use core::arch::{asm, global_asm};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.S"));

/// Initialize trap handling
pub fn init() {
    set_kernel_trap_entry();
}

/// 设置内核态陷入入口。
/// 将 `stvec` 指向内核陷入处理函数 `trap_from_kernel`，并使用 `TrapMode::Direct`，
/// 以确保在内核态执行期间发生的异常/中断由内核处理路径接管。
/// 通常在初始化或进入内核处理流程前调用；与 `set_user_trap_entry` 相对应。
fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

/// enable timer interrupt in supervisor mode
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

/// trap handler
/// 统一陷入处理函数（S态），不返回。
/// - 先设置内核陷入入口为内核路径，避免内核态再次陷入时误入 TRAMPOLINE；
/// - 读取 `scause`/`stval` 并按类型分派：
///   - 系统调用（UserEnvCall）：`sepc += 4` 跳过 `ecall`，记录次数，调用 `syscall(a7, [a0,a1,a2])`，将返回值写回 `a0`；
///   - 访存/取指异常：打印信息并终止当前任务（页故障返回码 -2）；
///   - 非法指令：终止当前任务（返回码 -3）；
///   - 监督级时钟中断：`set_next_trigger()` 预约下次中断，`suspend_current_and_run_next()` 抢占切换；
///   - 其他类型：`panic!`；
/// - 末尾调用 `trap_return()`，恢复用户上下文并通过 `__restore`/`sret` 返回到用户态。
#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    // trace!("into {:?}", scause.cause());
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            let current_task = current_task().unwrap();
            current_task.update_syscall_times(cx.x[17]);
            drop(current_task);
            // get system call return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] trap_handler:  {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    //println!("before trap_return");
    trap_return();
}

#[no_mangle]
/// 返回用户态（不返回）。
///
/// - 将 `stvec` 设置为用户态陷入入口（TRAMPOLINE）。
/// - 计算 TRAMPOLINE 页上 `__restore` 的虚拟地址并跳转。
/// - 跳转前设置寄存器：`a0` 为 TrapContext 的虚拟地址，`a1` 为用户页表 token（`satp`）。
/// - `__restore` 在汇编中恢复用户通用寄存器与相关 CSR（含 `sepc`），最终执行 `sret` 进入用户态入口。
pub fn trap_return() -> ! {
    set_user_trap_entry(); // 将 stvec 设置为用户态陷入入口（TRAMPOLINE）
    let trap_cx_ptr = TRAP_CONTEXT_BASE; // TrapContext 的虚拟地址，作为 __restore 的 a0
    let user_satp = current_user_token(); // 当前任务的用户页表 token（satp），作为 __restore 的 a1
    // 外部汇编符号：用于计算 __restore 在 TRAMPOLINE 上的虚拟地址
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE; // 通过偏移将 __restore 放到 TRAMPOLINE 的虚拟地址
    // trace!("[kernel] trap_return: ..before return");
    unsafe {
        asm!(
            "fence.i",                 // 同步 I-cache，确保 TRAMPOLINE 上的代码可见
            "jr {restore_va}",         // 跳转到 TRAMPOLINE 上的 __restore
            restore_va = in(reg) restore_va, // 传入跳转目标地址
            in("a0") trap_cx_ptr,      // a0 = TrapContext 虚拟地址
            in("a1") user_satp,        // a1 = 用户页表 token（satp）
            options(noreturn)           // 不返回：__restore 将恢复现场并 sret 进入 U 态
        );
    }
}

#[no_mangle]
/// handle trap from kernel
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn trap_from_kernel() -> ! {
    use riscv::register::sepc;
    trace!("stval = {:#x}, sepc = {:#x}", stval::read(), sepc::read());
    panic!("a trap {:?} from kernel!", scause::read().cause());
}

pub use context::TrapContext;
