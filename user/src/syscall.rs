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
    const GETPID: usize = 172;
    const FORK: usize = 220;
    const EXEC: usize = 221;
    const WAITPID: usize = 260;
    const MMAP: usize = 270;
    const MUNMAP: usize = 271;
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

pub fn sys_read(fd: usize, buf: &mut [u8]) -> isize {
    syscall(Syscall::READ, [fd, buf.as_mut_ptr() as usize, buf.len()])
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

pub fn sys_getpid() -> isize {
    syscall(Syscall::GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(Syscall::FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(Syscall::EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    syscall(Syscall::WAITPID, [pid as usize, exit_code_ptr as usize, 0])
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    syscall(Syscall::MMAP, [start, len, port])
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    syscall(Syscall::MUNMAP, [start, len, 0])
}
