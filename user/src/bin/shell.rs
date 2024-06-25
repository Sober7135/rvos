#![no_std]
#![no_main]

use user_lib::{console::getchar, exec, fork, waitpid};
#[macro_use]
extern crate user_lib;
extern crate alloc;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

use alloc::string::String;

#[no_mangle]
unsafe fn main() -> i32 {
    println!("Shell");
    let mut line = String::new();
    print!("$ ");
    loop {
        let ch = getchar();
        match ch {
            LF | CR => {
                println!();
                if !line.is_empty() {
                    line.push('\0');
                    let pid = fork();
                    if pid == 0 {
                        if exec(line.as_str()) == -1 {
                            println!("Error when executing");
                            return -1;
                        }
                        unreachable!()
                    } else {
                        let mut exit_code = 0;
                        let exit_pid = waitpid(pid, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("Shell: Process {} exited with code {}", pid, exit_code);
                    }
                    line.clear();
                }
                print!("$ ");
            }
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }
            _ => {
                line.push(ch as char);
                print!("{}", ch as char);
            }
        }
    }
}
