use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;
use crate::config::TRAP_CONTEXT_BASE;
use crate::fs::{File, Stderr, Stdin, Stdout};
use super::{SignalFlags, TaskContext};
use crate::mm::{translated_refmut, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::sync::UPSafeCell;
use crate::task::action::SignalActions;
use crate::task::id::{pid_alloc, KernelStack, PidHandle, TaskUserRes};
use crate::task::process::ProcessControlBlock;
use crate::trap::{trap_handler, TrapContext};

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
}

pub struct TaskControlBlock {
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub trap_cx_ppn: PhysPageNum,
    pub exit_code: Option<i32>,
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(
            process.clone(),
            ustack_base,
            alloc_user_res,
        );
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = KernelStack::new();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe { UPSafeCell::new(TaskControlBlockInner {
                res: Some(res),
                task_status: TaskStatus::Ready,
                task_cx: TaskContext::goto_trap_return(kstack_top),
                trap_cx_ppn,
                exit_code: None,
            })},
        }
    }
    
    pub fn get_process(&self) -> Arc<ProcessControlBlock> {
        self.process.upgrade().unwrap()
    }
    
    pub fn get_user_token(&self) -> usize {
        self.get_process().inner_exclusive_access().get_user_token()
    }
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
}