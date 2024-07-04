use crate::{
    fs::{open_file, OpenFlags},
    mm::{transfer_byte_buffer, translate_str, UserBuffer},
    process::{
        mark_current_exit, mark_current_suspend, mmap, munmap,
        processor::{get_current_task, get_current_user_token, schedule},
    },
    timer::get_time_ms,
};
pub struct Syscall;

// https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/unistd.h
impl Syscall {
    const OPEN: usize = 56;
    const CLOSE: usize = 57;
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
        Syscall::OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        Syscall::CLOSE => sys_close(args[0]),
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

fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = get_current_task().unwrap();
    let token = get_current_user_token();
    let path = translate_str(token, path);

    if let Some(inode) = open_file(&path, OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner.lock();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

fn sys_close(fd: usize) -> isize {
    let task = get_current_task().unwrap();
    let mut inner = task.inner.lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let task = get_current_task().unwrap();
    let inner = task.inner.lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        drop(inner);
        file.read(UserBuffer::new(transfer_byte_buffer(buf, len))) as isize
    } else {
        -1
    }
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let task = get_current_task().unwrap();
    let inner = task.inner.lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.writable() {
            return -1;
        }
        drop(inner);
        file.write(UserBuffer::new(transfer_byte_buffer(buf, len))) as isize
    } else {
        -1
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
    // open file
    let token = get_current_user_token();
    let path = translate_str(token, path);
    if let Some(inode) = open_file(&path, OpenFlags::RDONLY) {
        let data = inode.read_all();
        get_current_task().unwrap().exec(&data)
    } else {
        -1
    }
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
