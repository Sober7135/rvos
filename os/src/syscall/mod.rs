use crate::{
    mm::copy_from_user,
    print,
    process::{mark_current_exit, mark_current_suspend, processor::schedule},
    timer::get_time_ms,
};
use log::info;
pub struct Syscall;

// https://github.com/torvalds/linux/blob/9b6de136b5f0158c60844f85286a593cb70fb364/include/uapi/asm-generic/unistd.h
impl Syscall {
    const READ: usize = 63;
    const WRITE: usize = 64;
    const EXIT: usize = 93;
    const YIELD: usize = 124;
    const GETTIME: usize = 169;
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
    info!("[kernel] Application exited with code {}", exit_code);
    // mark current task to exit and run next
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
