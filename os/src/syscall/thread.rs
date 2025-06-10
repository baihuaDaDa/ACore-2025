use alloc::sync::Arc;
use crate::mm::kernel_token;
use crate::task::{add_task, current_task, TaskControlBlock};
use crate::trap::{trap_handler, TrapContext};

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    let task = current_task().unwrap();
    let process = task.get_process();
    // create a new thread
    let new_task = Arc::new(TaskControlBlock::new(
        Arc::clone(&process),
        task.inner_exclusive_access().res.as_ref().unwrap().ustack_base,
        true,
    ));
    // add new task to scheduler
    add_task(Arc::clone(&new_task));
    let new_task_inner = new_task.inner_exclusive_access();
    let new_task_res = new_task_inner.res.as_ref().unwrap();
    let new_task_tid = new_task_res.tid;
    let mut process_inner = process.inner_exclusive_access();
    // add new thread to current process
    let tasks = &mut process_inner.tasks;
    while tasks.len() < new_task_tid + 1 {
        tasks.push(None);
    }
    tasks[new_task_tid] = Some(Arc::clone(&new_task));
    let new_task_trap_cx = new_task_inner.get_trap_cx();
    *new_task_trap_cx = TrapContext::app_init_context(
        entry,
        new_task_res.ustack_top(),
        kernel_token(),
        new_task.kstack.get_top(),
        trap_handler as usize,
    );
    (*new_task_trap_cx).x[10] = arg;
    new_task_tid as isize
}

pub fn sys_waittid(tid: usize) -> isize {
    let task = current_task().unwrap();
    let process = task.get_process();
    let task_inner = task.inner_exclusive_access();
    let mut process_inner = process.inner_exclusive_access();
    // a thread cannot wait for itself or a non-existing thread
    if task_inner.res.as_ref().unwrap().tid == tid || process_inner.tasks.len() <= tid {
        return -1;
    }
    let mut exit_code: Option<i32> = None;
    let waited_task = process_inner.tasks[tid].as_ref();
    if waited_task.is_some() {
        if let Some(waited_exit_code) = waited_task.unwrap().inner_exclusive_access().exit_code {
            exit_code = Some(waited_exit_code);
        }
    } else {
        // waited thread does not exist
        return -1;
    }
    if exit_code.is_some() {
        // dealloc the exited thread
        process_inner.tasks[tid] = None;
        exit_code.unwrap() as isize
    } else {
        // waited thread has not exited
        -2
    }
}