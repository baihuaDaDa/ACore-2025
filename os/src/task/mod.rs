mod context;
mod switch;
mod task;
mod pid;
mod manager;
mod processor;
mod action;
mod signal;

use alloc::sync::Arc;
use lazy_static::*;
pub use context::TaskContext;
pub use task::{TaskControlBlock, TaskStatus};
pub use processor::{run_tasks, schedule, take_current_task, current_task, current_user_token, current_trap_cx};
pub use manager::{add_task, pid2task};
pub use signal::{MAX_SIG, SignalFlags};
pub use action::{SignalAction, SignalActions};
use crate::fs::{open_file, OpenFlags};

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn suspend_current_and_run_next() {
    // There must be an application running
    let task = take_current_task().unwrap();
    // access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // stop exclusively accessing current PCB
    // push back to ready queue
    add_task(task);
    // jump tp scheduling cycle
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    // access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc
    // access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // stop exclusively accessing parent PCB
    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // stop exclusively accessing current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

pub fn check_signals_error_of_current() -> Option<(i32, &'static str)> {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive_access();
    task_inner.signals.check_error()
}

pub fn current_add_signal(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.signals |= signal;
}

fn call_kernel_signal_handler(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    match signal {
        SignalFlags::SIGSTOP => {
            task_inner.frozen = true;
            task_inner.signals ^= SignalFlags::SIGSTOP;
        }
        SignalFlags::SIGCONT => {
            if task_inner.signals.contains(SignalFlags::SIGCONT) {
                task_inner.signals ^= SignalFlags::SIGCONT;
                task_inner.frozen = false;
            }
        }
        _ => {
            task_inner.killed = true;
        }
    }
}

fn call_user_signal_handler(sig: usize, signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let handler = task_inner.signal_actions.table[sig].handler;
    if handler != 0 {
        // user handler
        // handler flag
        task_inner.handling_sig = sig as isize;
        task_inner.signals ^= signal;
        // backup trap frame
        let trap_cx = task_inner.get_trap_cx();
        task_inner.trap_cx_backup = Some(*trap_cx);
        // modify trap frame
        trap_cx.sepc = handler;
        // put args (a0)
        trap_cx.x[10] = sig;
    } else {
        // default action
        println!("[kernel] task/call_user_signal_handler: default action: ignore it or kill process");
    }
}

pub fn check_pending_signals() {
    for sig in 0..(MAX_SIG + 1) {
        let task = current_task().unwrap();
        let task_inner = task.inner_exclusive_access();
        let signal = SignalFlags::from_bits(1 << sig).unwrap();
        if task_inner.signals.contains(signal) && (!task_inner.signal_mask.contains(signal)) {
            let mut masked = true;
            let handling_sig = task_inner.handling_sig;
            if handling_sig == -1 {
                masked = false;
            } else {
                let handling_sig = handling_sig as usize;
                if !task_inner.signal_actions.table[handling_sig]
                    .mask
                    .contains(signal) {
                    masked = false;
                }
            }
            if !masked {
                drop(task_inner);
                drop(task);
                if signal == SignalFlags::SIGKILL
                    || signal == SignalFlags::SIGSTOP
                    || signal == SignalFlags::SIGCONT
                    || signal == SignalFlags::SIGDEF {
                    // signal is a kernel signal
                    call_kernel_signal_handler(signal);
                } else {
                    // signal is a user signal
                    call_user_signal_handler(sig, signal);
                }
            }
        }
    }
}

pub fn handle_signals() {
    loop {
        check_pending_signals();
        let (frozen, killed) = {
            let task = current_task().unwrap();
            let task_inner = task.inner_exclusive_access();
            (task_inner.frozen, task_inner.killed)
        };
        if !frozen || killed {
            break;
        }
        suspend_current_and_run_next();
    }
}