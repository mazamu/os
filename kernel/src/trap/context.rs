//! Implementation of [`TrapContext`]

use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
/// trap context structure containing sstatus, sepc and registers
pub struct TrapContext {
    /// General-Purpose Register x0-31
    pub x: [usize; 32],
    /// sstatus
    pub sstatus: Sstatus,//32
    /// sepc
    pub sepc: usize,//33，发生异常的地址
    /// Token of kernel address space
    pub kernel_satp: usize,//34
    /// Kernel stack pointer of the current application
    pub kernel_sp: usize,//35
    /// Virtual address of trap handler entry point in kernel
    pub trap_handler: usize,//36
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);//设置用户态
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp);
        cx
    }
}
