// src/debug.rs
use core::fmt::{self, Write, Arguments};

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            js_log(s.as_ptr(), s.len());
        }
        Ok(())
    }
}

extern "C" {
    pub fn js_log(ptr: *const u8, len: usize);
}

pub fn console_log(args: Arguments) {
    let mut console = Console;
    fmt::write(&mut console, args).unwrap();
}

#[macro_export]
macro_rules! printf {
    ($($arg:tt)*) => ({
        $crate::debug::console_log(format_args!($($arg)*));
    })
}

#[cfg_attr(not(any(feature = "std", test)), panic_handler)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    printf!("Panic occurred!");

    let mut console = Console;
    writeln!(console, "{}", info).unwrap();

    if let Some(location) = info.location() {
        writeln!(
            console,
            "Panic location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        )
        .unwrap();
    }

    loop {}
}
