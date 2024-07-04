use crate::{batch::run_next_app, print};
use log::info;

pub(crate) struct Syscall;

impl Syscall {
    const READ: usize = 63;
    const WRITE: usize = 64;
    const EXIT: usize = 93;
}

// a0-a2 for arguments, a7 for syscall id
// return in a0
pub(crate) fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        Syscall::WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::READ => unimplemented!(),
        Syscall::EXIT => sys_exit(args[0] as i32),
        _ => panic!("unsupport system call!!!"),
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

fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}\n", exit_code);
    run_next_app()
}
