use crate::{
    print,
    task::{mark_current_exited, mark_current_suspend, run_next_task},
};
use log::info;
pub(crate) enum Syscall {
    Read,
    Write,
    Exit,
    Yield,
}

// https://github.com/torvalds/linux/blob/9b6de136b5f0158c60844f85286a593cb70fb364/include/uapi/asm-generic/unistd.h
impl Syscall {
    #[allow(unused)]
    fn value(&self) -> usize {
        match *self {
            Self::Read => 63,
            Self::Write => 64,
            Self::Exit => 93,
            Self::Yield => 124,
        }
    }
}

impl From<usize> for Syscall {
    fn from(value: usize) -> Self {
        match value {
            63 => Self::Read,
            64 => Self::Write,
            93 => Self::Exit,
            124 => Self::Yield,
            _ => {
                panic!("Unsupported syscall!")
            }
        }
    }
}

// a0-a2 for arguments, a7 for syscall id
// return in a0
pub(crate) fn syscall(id: Syscall, args: [usize; 3]) -> isize {
    match id {
        Syscall::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::Read => unimplemented!(),
        Syscall::Exit => sys_exit(args[0] as i32),
        Syscall::Yield => sys_yield(),
    }
}

const FD_STDOUT: usize = 1;
fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            unsafe {
                let slice = core::slice::from_raw_parts(buf, len);
                let str = core::str::from_utf8(slice).unwrap();
                print!("{}", str);
            }
            len as isize
        }
        _ => panic!("Unsupport fd"),
    }
}

fn sys_exit(exit_code: i32) -> isize {
    info!("[kernel] Application exited with code {}", exit_code);
    // mark current task to exit and run next
    mark_current_exited();
    run_next_task();
    unreachable!()
}

fn sys_yield() -> isize {
    mark_current_suspend();
    run_next_task();
    0
}
