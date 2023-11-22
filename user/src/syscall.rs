#![allow(unused)]

use core::arch::asm;

enum Syscall {
    Read,
    Write,
    Exit,
}

impl Syscall {
    pub(crate) fn value(&self) -> usize {
        match *self {
            Self::Read => 63,
            Self::Write => 64,
            Self::Exit => 93,
        }
    }
}

// a0-a2 for arguments, a7 for syscall id
// return in a0
fn syscall(id: Syscall, args: [usize; 3]) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a7") id.value(),
        )
    }
    ret
}

pub(crate) fn sys_write(fd: usize, buf: &[u8]) -> isize {
    syscall(Syscall::Write, [fd, buf.as_ptr() as usize, buf.len()])
}

pub(crate) fn sys_exit(state: i32) -> isize {
    syscall(Syscall::Exit, [state as usize, 0, 0])
}
