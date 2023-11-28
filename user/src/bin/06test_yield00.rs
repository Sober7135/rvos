#![no_std]
#![no_main]

use user_lib::yield_;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    for i in 0..10 {
        println!("[06test_yield00] i = {}", i);
        println!(">>>>>>>>>>>>>>>>>>>>>>>>06test_yield00 restore<<<<<<<<<<<<<<<<<<<<");
        yield_();
        println!(">>>>>>>>>>>>>>>>>>>>>>>>06test_yield00 restore<<<<<<<<<<<<<<<<<<<<");
    }
    0
}
