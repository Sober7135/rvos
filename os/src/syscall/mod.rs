use crate::{
    mm::copy_from_user,
    print,
    process::{
        mark_current_exit, mark_current_suspend,
        processor::{get_current_task, schedule},
    },
    timer::get_time_ms,
};
use log::{debug, info};
pub struct Syscall;

// https://github.com/torvalds/linux/blob/9b6de136b5f0158c60844f85286a593cb70fb364/include/uapi/asm-generic/unistd.h
impl Syscall {
    const READ: usize = 63;
    const WRITE: usize = 64;
    const EXIT: usize = 93;
    const YIELD: usize = 124;
    const GETTIME: usize = 169;
    const GETPID: usize = 172;
    const FORK: usize = 220; // clone ???
}

// a0-a2 for arguments, a7 for syscall id
// return in a0
pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        Syscall::WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::READ => unimplemented!(),
        Syscall::EXIT => sys_exit(args[0] as i32),
        Syscall::YIELD => sys_yield(),
        Syscall::GETTIME => sys_get_time(),
        Syscall::GETPID => sys_getpid(),
        Syscall::FORK => sys_fork(),
        _ => panic!("unsupport system call!!!"),
    }
}

const FD_STDOUT: usize = 1;

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffer = copy_from_user(buf, len);
            for buf in buffer {
                print!("{}", core::str::from_utf8(buf).unwrap());
            }
            len as isize
        }
        _ => panic!("Unsupport fd"),
    }
}

fn sys_exit(exit_code: i32) -> isize {
    // mark current task to exit and schedule
    mark_current_exit(exit_code);
    schedule();
    unreachable!()
}

fn sys_yield() -> isize {
    mark_current_suspend();
    schedule();
    0
}

fn sys_get_time() -> isize {
    get_time_ms() as isize
}

fn sys_getpid() -> isize {
    get_current_task().unwrap().get_pid() as isize
}

fn sys_fork() -> isize {
    let current = get_current_task().unwrap();

    let child = current.fork();
    let child_pid = child.get_pid();
    let child_inner = child.inner.lock();
    child_inner.get_trap_context().x[10] = 0;

    child_pid as isize
}
