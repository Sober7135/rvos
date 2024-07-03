use crate::{fs::Stdout, sbi};
use core::fmt::{self, Write};

impl Write for Stdout {
    fn write_str(&mut self, fmt: &str) -> fmt::Result {
        for c in fmt.chars() {
            sbi::console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg: tt)*) => {
        $crate::console::print(format_args!($($arg)?))
    };
}

#[macro_export]
macro_rules! println {
    () => {
      $crate::print!("\n")
    };
    ($($arg: tt)*) => {
        $crate::console::print(format_args_nl!($($arg)*))
    };
}
