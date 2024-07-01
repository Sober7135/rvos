#![no_std]
#![no_main]

use core::arch::asm;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Try to excute privileged instruction in U Mode");
    println!("Kernel should kill this program!");
    unsafe { asm!("sret") }
    0
}
