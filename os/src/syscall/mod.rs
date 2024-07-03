use crate::{
    mm::transfer_byte_buffer,
    print,
    process::{
        mark_current_exit, mark_current_suspend, mmap, munmap,
        processor::{get_current_task, schedule},
    },
    sbi::console_getchar,
    timer::get_time_ms,
};
pub struct Syscall;

// https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/unistd.h
impl Syscall {
    const READ: usize = 63;
    const WRITE: usize = 64;
    const EXIT: usize = 93;
    const YIELD: usize = 124;
    const GETTIME: usize = 169;
    const GETPID: usize = 172;
    const FORK: usize = 220; // clone ???
    const EXEC: usize = 221;
    const WAITPID: usize = 260; // wait4 in unistd.h
    const MMAP: usize = 270;
    const MUNMAP: usize = 271;
}

// a0-a2 for arguments, a7 for syscall id
// return in a0
pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        Syscall::WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::READ => sys_read(args[0], args[1] as *const u8, args[2]),
        Syscall::EXIT => sys_exit(args[0] as i32),
        Syscall::YIELD => sys_yield(),
        Syscall::GETTIME => sys_get_time(),
        Syscall::GETPID => sys_getpid(),
        Syscall::FORK => sys_fork(),
        Syscall::EXEC => sys_exec(args[0] as *const u8),
        Syscall::WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        Syscall::MMAP => sys_mmap(args[0], args[1], args[2]),
        Syscall::MUNMAP => sys_munmap(args[0], args[1]),

        _ => panic!("unsupport system call!!!"),
    }
}

const FD_STDOUT: usize = 1;
const FD_STDIN: usize = 0;

fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "current we only support one character");
            let mut c;
            loop {
                c = console_getchar();
                if c == 0 {
                    mark_current_suspend();
                    schedule();
                } else {
                    break;
                }
            }
            let ch = c as u8;
            unsafe {
                transfer_byte_buffer(buf, len)
                    .get_unchecked_mut(0)
                    .as_mut_ptr()
                    .write_volatile(ch)
            }
            1
        }
        _ => panic!("current we don't support other fd"),
    }
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffer = transfer_byte_buffer(buf, len);
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

fn sys_exec(path: *const u8) -> isize {
    get_current_task().unwrap().exec(path)
}

// we dont support option
fn sys_waitpid(pid: isize, wstatus: *mut i32) -> isize {
    get_current_task().unwrap().waitpid(pid, wstatus)
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if mmap(start, len, port).is_ok() {
        0
    } else {
        -1
    }
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    if munmap(start, len).is_ok() {
        0
    } else {
        -1
    }
}
