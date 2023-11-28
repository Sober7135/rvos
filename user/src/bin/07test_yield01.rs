#![no_std]
#![no_main]

use user_lib::yield_;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    for i in 0..10 {
        println!("[07test_yield01] i = {}", i);
        println!(">>>>>>>>>>>>>>>>>>>>>>>>07test_yield01 yield<<<<<<<<<<<<<<<<<<<<");
        yield_();
        println!(">>>>>>>>>>>>>>>>>>>>>>>>07test_yield01 restore<<<<<<<<<<<<<<<<<<<<");
    }
    0
}
