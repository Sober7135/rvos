#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exec, fork, waitpid};

// name exit_code
const APPS: &[(&str, i32)] = &[
    ("fork_test\0", 0),
    ("fork_test2\0", 0),
    ("matrix\0", 0),
    ("power_3\0", 0),
    ("power_5\0", 0),
    ("power_7\0", 0),
    ("priv_csr\0", -1),
    ("priv_inst\0", -1),
    ("sleep\0", 0),
    ("store_fault\0", -1),
    ("mmap\0", 0),
    ("mmap2\0", -1),
    ("mmap3\0", 0),
];

#[no_mangle]
fn main() -> i32 {
    let mut passed = 0;
    for (app, code) in APPS {
        println!("[{}] starting running {}\n", file!(), app);
        let pid = fork();
        if pid == 0 {
            exec(app);
            unreachable!();
        } else {
            let mut exit_code = 0;
            let wait_pid = waitpid(pid, &mut exit_code);
            assert_eq!(wait_pid, pid);
            println!(
                "\n[{}] Test {} exited with code {}",
                file!(),
                app,
                exit_code
            );
            if exit_code == *code {
                passed += 1;
                println!("[{}] Test {} OK!", file!(), app);
            } else {
                println!("[{}] Test {} Failed!", file!(), app);
            }
        }
        println!();
    }
    if passed == APPS.len() {
        println!("[{}] All Tests OK!", file!());
    } else {
        println!("[{}] {}/{} Tests OK!", file!(), passed, APPS.len());
    }
    0
}
