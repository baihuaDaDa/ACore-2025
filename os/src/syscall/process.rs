use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_ref, translated_refmut, translated_str};
use crate::task::{suspend_current_and_run_next, exit_current_and_run_next, current_task, add_task, current_user_token, SignalFlags, SignalAction, MAX_SIG, pid2process};
use crate::timer::get_time_ms;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Thread exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_kill(pid: usize, signum: i32) -> isize {
    if let Some(task) = pid2process(pid) {
        if let Some(flag) = SignalFlags::from_bits(1 << signum) {
            // insert the signal if legal
            let mut task_ref = task.inner_exclusive_access();
            if task_ref.signals.contains(flag) {
                return -1;
            }
            task_ref.signals.insert(flag);
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

fn check_sigaction_error(signal: SignalFlags, action: usize, old_action: usize) -> bool {
    if action == 0
        || old_action == 0
        || signal == SignalFlags::SIGKILL
        || signal == SignalFlags::SIGSTOP {
        true
    } else {
        false
    }
}

pub fn sys_sigaction(
    signum: i32,
    action: *const SignalAction,
    old_action: *mut SignalAction,
) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let process = task.get_process();
    let mut process_inner = process.inner_exclusive_access();
    if signum as usize > MAX_SIG {
        return -1;
    }
    if let Some(flag) = SignalFlags::from_bits(1 << signum) {
        if check_sigaction_error(flag, action as usize, old_action as usize) {
            return -1;
        }
        let prev_action = process_inner.signal_actions.table[signum as usize];
        *translated_refmut(token, old_action) = prev_action;
        process_inner.signal_actions.table[signum as usize] = *translated_ref(token, action);
        0
    } else {
        -1
    }
}

pub fn sys_sigprocmask(mask: u32) -> isize {
    if let Some(task) = current_task() {
        let process = task.get_process();
        let mut process_inner = process.inner_exclusive_access();
        let old_mask = process_inner.signal_mask;
        if let Some(flag) = SignalFlags::from_bits(mask) {
            process_inner.signal_mask = flag;
            old_mask.bits() as isize
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_sigreturn() -> isize {
    if let Some(task) = current_task() {
        let process = task.get_process();
        let mut process_inner = process.inner_exclusive_access();
        process_inner.handling_sig = -1;
        // restore the trap context
        let trap_cx = task.inner_exclusive_access().get_trap_cx();
        *trap_cx = process_inner.trap_cx_backup.unwrap();
        trap_cx.x[10] as isize
    } else {
        -1
    }
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().get_process().getpid() as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_process = current_task.get_process().fork();
    let new_process_inner = new_process.inner_exclusive_access();
    let new_pid = new_process.getpid();
    let new_main_thread = new_process_inner
        .tasks[0]
        .as_ref()
        .unwrap();
    let trap_cx = new_main_thread
        .inner_exclusive_access()
        .get_trap_cx();
    // for child process, fork returns 0
    trap_cx.x[10] = 0; // x[10] is a0 reg
    // add new task to scheduler
    new_pid as isize
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe { args = args.add(1); }
    }
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        let argc = args_vec.len();
        task.get_process().exec(all_data.as_slice(), args_vec);
        argc as isize
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process
    // access current PCB exclusively
    let process = task.get_process();
    let mut process_inner = process.inner_exclusive_access();
    if process_inner.children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none() {
        return -1;
        // stop exclusively accessing current PCB
    }
    let pair = process_inner.children
        .iter()
        .enumerate()
        .find(|(_, p)| {
            p.inner_exclusive_access().is_zombie && (pid == -1 || pid as usize == p.getpid())
        });
    if let Some((idx, _)) = pair {
        let child = process_inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        let exit_code = child.inner_exclusive_access().exit_code;
        *translated_refmut(process_inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}