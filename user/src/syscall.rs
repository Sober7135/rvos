#![allow(unused)]

use core::arch::asm;

// https://github.com/torvalds/linux/blob/9b6de136b5f0158c60844f85286a593cb70fb364/include/uapi/asm-generic/unistd.h
struct Syscall;

impl Syscall {
    const READ: usize = 63;
    const WRITE: usize = 64;
    const EXIT: usize = 93;
    const YIELD: usize = 124;
    const GETTIME: usize = 169;
}

// a0-a2 for arguments, a7 for syscall id
// return in a0
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a7") id,
        )
    }
    ret
}

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    syscall(Syscall::WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}

pub fn sys_exit(state: i32) -> isize {
    syscall(Syscall::EXIT, [state as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(Syscall::YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(Syscall::GETTIME, [0, 0, 0])
}
