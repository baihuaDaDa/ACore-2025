use core::arch::{asm, global_asm};
use riscv::register::{mtvec::TrapMode, scause::{self, Exception, Interrupt, Trap}, sie, stval, stvec, sip};
use crate::syscall::syscall;
use crate::task::{check_signals_error_of_current, current_add_signal, current_trap_cx, current_trap_cx_user_va, current_user_token, exit_current_and_run_next, handle_signals, suspend_current_and_run_next, SignalFlags};

mod context;

pub use context::TrapContext;
use crate::config::TRAMPOLINE;
use crate::timer::check_timer;

global_asm!(include_str!("trap.S"));

pub fn init() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
        sie::set_stimer();
        sie::set_sext();
        sie::set_ssoft();
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

#[unsafe(no_mangle)]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = current_trap_cx_user_va();
    let user_satp = current_user_token();
    unsafe extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
        );
    }
    panic!("Unreachable in back_to_user!");
}

#[unsafe(no_mangle)]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let mut cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) => {
            println!(
                "[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                stval, cx.sepc
            );
            // exit_current_and_run_next(-2);
            current_add_signal(SignalFlags::SIGSEGV);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!(
                "[kernel] IllegalInstruction in application, bad instruction = {:#x}, kernel killed it.",
                cx.sepc
            );
            // exit_current_and_run_next(-3);
            current_add_signal(SignalFlags::SIGILL);
        }
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            // println!("[kernel] Timer interrupt! pid: {}", current_task().unwrap().get_process().getpid());
            let sip = sip::read().bits();
            unsafe {
                asm! {"csrw sip, {sip}", sip = in(reg) sip ^ 2};
            } // clear the Supervisor Software Interrupt bit
            check_timer();
            suspend_current_and_run_next();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}", scause.cause(), stval);
        }
    }
    handle_signals();
    // check error signals (if error then exit)
    if let Some((errno, msg)) = check_signals_error_of_current() {
        println!("[kernel] {}", msg);
        exit_current_and_run_next(errno);
    }
    trap_return();
}