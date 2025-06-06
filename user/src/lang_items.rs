use core::panic::PanicInfo;
use crate::{getpid, kill, SIGABRT};

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        crate::println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        crate::println!("Panicked: {}", info.message());
    }
    kill(getpid() as usize, SIGABRT);
    unreachable!()
}
