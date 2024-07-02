#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork, get_time, getpid, sleep, wait};

const MAX_CHILD: usize = 30;

#[no_mangle]
fn main() -> i32 {
    for i in 0..MAX_CHILD {
        let pid = fork();
        if pid == 0 {
            let current = get_time();
            let sleep_len = current * current % 1000 + 300;
            println!(
                "I am child {}, pid={}, sleeping {}ms",
                i,
                getpid(),
                sleep_len
            );
            sleep(sleep_len as usize);
            println!("pid {} wake up!", getpid());
            exit(0);
        } else {
            println!("forked child {} pid={}", i, pid);
        }
        assert!(pid > 0);
    }
    let mut exit_code = 0;
    for _ in 0..MAX_CHILD {
        let wait_pid = wait(&mut exit_code);
        assert_eq!(exit_code, 0);
        println!("pid={} exited with code {}", wait_pid, exit_code);
    }
    if wait(&mut exit_code) > 0 {
        panic!("wait too many");
    }
    0
}
