OUTPUT_ARCH(riscv)            /* 指定目标体系结构为 RISC-V */
ENTRY(_start)                 /* 指定程序入口符号：链接后 PC 从 _start 开始执行 */
BASE_ADDRESS = 0x80200000;    /* 内核镜像的加载基地址（物理/虚拟直映的起点） */

SECTIONS                     /* 定义各段在最终镜像中的布局 */
{
    . = BASE_ADDRESS;        /* 将当前位置（location counter，符号 .）设置为基地址 */
    skernel = .;             /* 记录内核镜像起始地址符号 */

    stext = .;               /* 记录 .text 段起始地址 */
    .text : {                /* 代码段：放置可执行指令 */
        *(.text.entry)       /* 入口代码段，包含 _start 等启动入口 */
        . = ALIGN(4K);       /* 4KiB 对齐，便于分页映射 */
        strampoline = .;     /* 记录 TRAMPOLINE 物理地址（strampoline 符号） */
        *(.text.trampoline); /* 放置 TRAMPOLINE 的汇编粘合代码（trap 入口/出口） */
        . = ALIGN(4K);       /* 再次按页对齐，分隔 trampoline 与普通代码 */
        *(.text .text.*)     /* 其余普通代码段 */
    }

    . = ALIGN(4K);           /* 结束 .text 后页对齐 */
    etext = .;               /* 记录 .text 段结束地址 */
    srodata = .;             /* 记录 .rodata 段起始地址 */
    .rodata : {              /* 只读数据段：常量、字符串等 */
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);           /* 结束 .rodata 后页对齐 */
    erodata = .;             /* 记录 .rodata 段结束地址 */
    sdata = .;               /* 记录 .data 段起始地址 */
    .data : {                /* 可写数据段：已初始化的全局/静态变量 */
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);           /* 结束 .data 后页对齐 */
    edata = .;               /* 记录 .data 段结束地址 */
    sbss_with_stack = .;     /* 记录包含内核栈的 .bss 起点（先放栈） */
    .bss : {                 /* BSS 段：未初始化的全局/静态变量与内核栈 */
        *(.bss.stack)        /* 先放置内核栈（位于 BSS 顶部，便于标记与隔离） */
        sbss = .;            /* 记录常规 BSS 的起点 */
        *(.bss .bss.*)       /* 常规 BSS 变量 */
        *(.sbss .sbss.*)     /* 小型 BSS（small BSS） */
    }

    . = ALIGN(4K);           /* 结束 .bss 后页对齐 */
    ebss = .;                /* 记录 .bss 段结束地址 */
    ekernel = .;             /* 记录内核镜像结束地址符号 */

    /DISCARD/ : {            /* 丢弃不需要的调试/异常处理元数据，减小镜像体积 */
        *(.eh_frame)
    }
}