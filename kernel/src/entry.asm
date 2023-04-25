    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top #load address,把boot_stack_top加载到sp
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top: