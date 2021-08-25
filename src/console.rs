use core::fmt::{self, Write};
use crate::sbi;

static mut KERNEL_LOG: bool = true ;
pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s : &str) -> fmt::Result {
        for c in s.chars() {
            sbi::sbi_call(sbi::PUT_CHAR, [c as usize, 0, 0]);
        }
        Ok(())
    }
}
impl Stdout {
    pub fn is_log() -> bool {
        return  unsafe { KERNEL_LOG }  
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

pub fn turn_off_log() {
    unsafe { KERNEL_LOG = false; };
}
pub fn turn_on_log() {
    unsafe { KERNEL_LOG = true; };
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! log{
    (@inner_print $fmt: literal, $(, $($arg:tt)+)?) => {
        if $crate::console::Stdout::is_log() {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
        $crate::console::print(format_args!("\x1b[0m"));
        }
    };
    (info $fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::Stdout::is_log() {
        $crate::console::print(format_args!("\x1b[0;34m[Info]: "));
        log!(@inner_print $fmt, $(, $($arg)+)?);
        }
    };
    (error $fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::Stdout::is_log() {
        $crate::console::print(format_args!("\x1b[0;31m[Error]: "));
        log!(@inner_print $fmt, $(, $($arg)+)?);
        }
    };
    (warn $fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::Stdout::is_log() {
        $crate::console::print(format_args!("\x1b[0;93m[Warn]: "));
        log!(@inner_print $fmt, $(, $($arg)+)?);
        }
    };
    (debug $fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::Stdout::is_log() {
        $crate::console::print(format_args!("\x1b[0;32m[Debug]: "));
        log!(@inner_print $fmt, $(, $($arg)+)?);
        }
    };
    (trace $fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::Stdout::is_log() {
        $crate::console::print(format_args!("\x1b[0;90m[Trace]: "));
        log!(@inner_print $fmt, $(, $($arg)+)?);
        }
    }
}
