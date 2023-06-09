//! Implementation of process management mechanism
//!
//! Here is the entry for process scheduling required by other modules
//! (such as syscall or clock interrupt).
//! By suspending or exiting the current process, you can
//! modify the process state, manage the process queue through TASK_MANAGER,
//! and switch the control flow through PROCESSOR.
//!
//! Be careful when you see [`__switch`]. Control flow around this function
//! might not be what you expect.

mod context;
mod manager;
mod pid;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use alloc::sync::Arc;
use lazy_static::*;
use manager::fetch_task;
use switch::__switch;
use crate::mm::VirtAddr;
use crate::mm::MapPermission;
use crate::config::PAGE_SIZE;
use crate::timer::get_time_us;
pub use crate::syscall::process::TaskInfo;
use crate::fs::{open_file, OpenFlags};
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;
pub use manager::add_task;
pub use pid::{pid_alloc, KernelStack, PidHandle};
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
};
use crate::config::BIG_STRIDE;
use crate::config::LOG;
/// Make current task suspended and switch to the next task
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// Exit current task, recycle process resources and switch to the next task
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    /// Creation of initial process
    ///
    /// the name "initproc" may be changed to any other app name like "usertests",
    /// but we have user_shell, so we don't need to change it.
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
    
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn get_task_info() -> TaskInfo {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    let new_info = TaskInfo {
        status: inner.task_status,
        syscall_times: inner.syscall_times,
        time: get_time_us() / 1000 - inner.start_time, 
    };
    drop(inner);
    new_info
}

pub fn update_task_info(syscall_id: usize) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.syscall_times[syscall_id] += 1;
    drop(inner)
}

pub fn mmap(start: usize, len: usize, prot: usize) -> isize {
trace!("{}{}", module_path!(), "::mmap");
    
    if len == 0 {
        return 0;
    }
    if (prot >> 3) != 0 || (prot & 0x7) == 0 || start % 4096 != 0 {
        return -1;
    } 
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let memory_set = &mut inner.memory_set;
    let l: VirtAddr = start.into();
    let r: VirtAddr = (start + len).into();
    let lvpn = l.floor();
    let rvpn = r.ceil();
    for area in &memory_set.areas {
        //要申请的页不能覆盖已有的逻辑段已经申请的虚拟页地址，否则错误
        if lvpn <= area.vpn_range.get_start() && rvpn > area.vpn_range.get_start() {
            return -1;
        }
    }
    let mut permission = MapPermission::from_bits((prot as u8) << 1).unwrap();
    permission.set(MapPermission::U, true);
    let mut start = start;
    let end = start + len;
    while start < end {
        let mut endr = start + PAGE_SIZE;
        if endr > end {
            endr = end;
        }
        memory_set.insert_framed_area(start.into(), endr.into(), permission);
        start += PAGE_SIZE;
    }
    0
}

pub fn munmap(start: usize, len: usize) -> isize {
trace!("{}{}", module_path!(), "::munmap");
    if len == 0{
        return 0;
    }
    if start % 4096 != 0 {
        return -1;
    }
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let memory_set = &mut inner.memory_set;
    let l: VirtAddr = start.into();
    let r: VirtAddr = (start + len).into();
    let lvpn = l.floor();
    let rvpn = r.ceil();
    let mut cnt = 0;
    for area in &memory_set.areas {
        if lvpn <= area.vpn_range.get_start() && rvpn > area.vpn_range.get_start() {
            cnt += 1;
        }
    }
    if cnt < rvpn.0 - lvpn.0 {
        return -1;
    }
    for i in 0..memory_set.areas.len() {
        if !memory_set.areas.get(i).is_some() {
            continue;
        }
        if lvpn <= memory_set.areas[i].vpn_range.get_start() && rvpn > memory_set.areas[i].vpn_range.get_start() {
            memory_set.areas[i].unmap(&mut memory_set.page_table);
            memory_set.areas.remove(i);
        }
    }
    
    0
}
pub fn set_priority(prio: usize) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_priority = prio;
    task_inner.task_stride = BIG_STRIDE / prio;
    drop(task_inner);
}