use log::info;
use crate::{batch::run_next_app, print};
pub(crate) enum Syscall {
    Read,
    Write,
    Exit,
}

impl Syscall {
    #[allow(unused)]
    fn value(&self) -> usize {
        match *self {
            Self::Read => 63,
            Self::Write => 64,
            Self::Exit => 93,
        }
    }
}

impl From<usize> for Syscall {
    fn from(value: usize) -> Self {
        match value {
            63 => Self::Read,
            64 => Self::Write,
            93 => Self::Exit,
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
    info!("[kernel] Application exited with code {}", exit_code);
    run_next_app()
}
