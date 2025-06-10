use crate::fs::{make_pipe, open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let process = task.get_process();
    let mut process_inner = process.inner_exclusive_access();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = process_inner.alloc_fd();
    process_inner.fd_table[new_fd] = process_inner.fd_table[fd].clone();
    new_fd as isize
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let process = task.get_process();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut process_inner = process.inner_exclusive_access();
        let fd = process_inner.alloc_fd();
        process_inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let process = task.get_process();
    let mut process_inner = process.inner_exclusive_access();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        return -1;
    }
    process_inner.fd_table[fd] = None;
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let process = task.get_process();
    let token = current_user_token();
    let mut process_inner = process.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = process_inner.alloc_fd();
    process_inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = process_inner.alloc_fd();
    process_inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let process = task.get_process();
    let process_inner = process.inner_exclusive_access();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &process_inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(process_inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let process = task.get_process();
    let process_inner = process.inner_exclusive_access();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &process_inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(process_inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}