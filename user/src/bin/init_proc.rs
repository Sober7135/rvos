#![no_std]
#![no_main]

use user_lib::{exec, fork, wait};

#[macro_use]
extern crate user_lib;

#[no_mangle]
unsafe fn main() -> i32 {
    let pid = fork();
    if pid == 0 {
        exec("shell\0");
    } else {
        loop {
            let mut exit_code = 0;
            let pid = wait(&mut exit_code);
            if pid < 0 {
                continue;
            }
            println!(
                "[init] Released a zombie process, pid={}, exit_code={}",
                pid, exit_code
            );
        }
    }

    0
}
