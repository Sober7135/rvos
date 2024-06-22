use super::*;
use core::fmt::{self, Write};

const STDIN: usize = 0;
const STDOUT: usize = 1;
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, fmt: &str) -> fmt::Result {
        write(STDOUT, fmt.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($($arg: tt)+)*) => {
        $crate::console::print(format_args!($($($arg)+)?))
    };
}

#[macro_export]
#[allow_internal_unstable(format_args_nl)]
macro_rules! println {
    ($($arg: tt)*) => {
        $crate::console::print(format_args_nl!($($arg)*))
    };
}

pub fn getchar() -> u8 {
    let mut ch = [0u8; 1];
    read(STDIN, &mut ch);
    unsafe { *ch.get_unchecked(0) }
}
