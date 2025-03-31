use crate::loader::run_next_app;

pub fn sys_exit(xstate: i32) -> ! {
    println!("[kernel] Application exited with code {}", xstate);
    run_next_app();
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}