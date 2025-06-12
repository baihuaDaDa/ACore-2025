mod context;
mod switch;
mod task;
mod id;
mod manager;
mod processor;
mod action;
mod signal;
mod process;

use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
pub use context::TaskContext;
pub use task::{TaskControlBlock, TaskStatus};
pub use processor::{run_tasks, schedule, take_current_task, current_task, current_user_token, current_trap_cx, current_process, current_trap_cx_user_va, current_kstack_top};
pub use manager::{add_task, wakeup_task, pid2process, remove_from_pid2process};
pub use signal::{MAX_SIG, SignalFlags};
pub use action::{SignalAction, SignalActions};
use crate::config::INIT_PROC;
use crate::fs::{open_file, OpenFlags};
use crate::sbi::shutdown;
use crate::task::id::TaskUserRes;
use crate::task::manager::remove_task;
use crate::task::process::ProcessControlBlock;
use crate::timer::remove_timer;

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = {
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        ProcessControlBlock::new(v.as_slice())
    };
}

pub fn add_initproc() {
    add_task(Arc::clone(INITPROC.inner_exclusive_access().tasks[0].as_ref().unwrap()));
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
    // stop exclusively accessing current TCB
    // push back to ready queue
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn block_current_and_run_next() {
    // There must be an application running
    let task = take_current_task().unwrap();
    // access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // change status to Ready
    task_inner.task_status = TaskStatus::Blocked;
    drop(task_inner);
    // stop exclusively accessing current TCB
    // do not push back to ready queue
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    // access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let process = task.get_process();
    let tid = task_inner.res.as_ref().unwrap().tid;
    // record exit code
    task_inner.exit_code = Some(exit_code);
    task_inner.res = None;
    // here we do not remove the thread since we are still using the kstack
    // it will be deallocated when sys_waittid is called
    drop(task_inner);
    drop(task);
    // terminate the process if this is the main thread
    if tid == 0 {
        let pid = process.getpid();
        if pid == INIT_PROC {
            println!("[kernel] Init process exit with exit_code {}.", exit_code);
            if exit_code != 0 {
                shutdown(true);
            } else {
                shutdown(false);
            }
        }
        // remove PCB from PID2TCB
        remove_from_pid2process(pid);
        let mut process_inner = process.inner_exclusive_access();
        // change status to Zombie
        process_inner.is_zombie = true;
        // record exit code of main process
        process_inner.exit_code = exit_code;
        // do not move to its parent but under initproc
        {
            let mut initproc_inner = INITPROC.inner_exclusive_access();
            for child in process_inner.children.iter() {
                child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
            process_inner.children.clear();
        }
        // deallocate user res (including tid/trap_cx/ustack) of all threads
        // it has to be done before we dealloc the whole memory_set
        // otherwise they will be deallocated twice
        let mut recycle_res = Vec::<TaskUserRes>::new();
        for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            remove_inactive_task(Arc::clone(&task));
            let mut task_inner = task.inner_exclusive_access();
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
        }
        // release process_inner first to let drop(res) access PCB inner
        drop(process_inner);
        recycle_res.clear();
        let mut process_inner = process.inner_exclusive_access();
        // deallocate other data in user space
        process_inner.memory_set.recycle_data_pages();
        // drop file descriptors
        process_inner.fd_table.clear();
        // Remove all tasks except for the main thread itself
        while process_inner.tasks.len() > 1 {
            process_inner.tasks.pop();
        }
    }
    drop(process);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

fn remove_inactive_task(task: Arc<TaskControlBlock>) {
    remove_task(Arc::clone(&task));
    remove_timer(Arc::clone(&task));
}

pub fn check_signals_error_of_current() -> Option<(i32, &'static str)> {
    let task = current_task().unwrap();
    let process = task.get_process();
    let process_inner = process.inner_exclusive_access();
    process_inner.signals.check_error()
}

pub fn current_add_signal(signal: SignalFlags) {
    let task = current_task().unwrap();
    let process = task.get_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.signals |= signal;
}

fn call_kernel_signal_handler(signal: SignalFlags) {
    let task = current_task().unwrap();
    let process = task.get_process();
    let mut process_inner = process.inner_exclusive_access();
    match signal {
        SignalFlags::SIGSTOP => {
            process_inner.frozen = true;
            process_inner.signals ^= SignalFlags::SIGSTOP;
        }
        SignalFlags::SIGCONT => {
            if process_inner.signals.contains(SignalFlags::SIGCONT) {
                process_inner.signals ^= SignalFlags::SIGCONT;
                process_inner.frozen = false;
            }
        }
        _ => {
            process_inner.killed = true;
        }
    }
}

fn call_user_signal_handler(sig: usize, signal: SignalFlags) {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive_access();
    let process = task.get_process();
    let mut process_inner = process.inner_exclusive_access();
    let handler = process_inner.signal_actions.table[sig].handler;
    if handler != 0 {
        // user handler
        // handler flag
        process_inner.handling_sig = sig as isize;
        process_inner.signals ^= signal;
        // backup trap frame
        let trap_cx = task_inner.get_trap_cx();
        process_inner.trap_cx_backup = Some(*trap_cx);
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
        let process = task.get_process();
        let process_inner = process.inner_exclusive_access();
        let signal = SignalFlags::from_bits(1 << sig).unwrap();
        if process_inner.signals.contains(signal) && (!process_inner.signal_mask.contains(signal)) {
            let mut masked = true;
            let handling_sig = process_inner.handling_sig;
            if handling_sig == -1 {
                masked = false;
            } else {
                let handling_sig = handling_sig as usize;
                if !process_inner.signal_actions.table[handling_sig]
                    .mask
                    .contains(signal) {
                    masked = false;
                }
            }
            if !masked {
                drop(process_inner);
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
            let process = task.get_process();
            let process_inner = process.inner_exclusive_access();
            (process_inner.frozen, process_inner.killed)
        };
        if !frozen || killed {
            break;
        }
        suspend_current_and_run_next();
    }
}