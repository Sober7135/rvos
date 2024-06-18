#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
unsafe fn main() -> i32 {
    println!("this is init_proc");
    println!("starting loopping");
    0
}
