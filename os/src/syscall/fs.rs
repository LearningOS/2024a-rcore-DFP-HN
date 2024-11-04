//! File and filesystem-related syscalls

use crate::fs::{open_file, OpenFlags, Stat, StatMode, ROOT_INODE};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.0.clone();
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.0.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some((inode, path));
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    use crate::mm::{VirtAddr, PageTable};
    let task= current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if _fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[_fd] {
        let name = file.1.clone();
        drop(inner);
        let (ino, is_file, nlink) = ROOT_INODE.get_fstat(name.as_str());
        let token = current_user_token();
        let virt_dev = VirtAddr(_st as usize);
        let page_table = PageTable::from_token(token);
        if let Some(phy_dev) = page_table.translate_va(virt_dev) {
            let phy_dev = phy_dev.0 as *mut u64;
            unsafe {
                *phy_dev = 0;
            }
        }
        else {
            return -1;
        }
        let virt_ino = VirtAddr(_st as usize + 8);
        let page_table = PageTable::from_token(token);
        if let Some(phy_ino) = page_table.translate_va(virt_ino) {
            let phy_ino = phy_ino.0 as *mut u64;
            unsafe {
                *phy_ino = ino;
            }
        }
        else {
            return -1;
        }
        let virt_mode = VirtAddr(_st as usize + 16);
        let page_table = PageTable::from_token(token);
        if let Some(phy_mode) = page_table.translate_va(virt_mode) {
            let phy_mode = phy_mode.0 as *mut StatMode;
            unsafe {
                if is_file {
                    *phy_mode = StatMode::FILE;
                }
                else {
                    *phy_mode = StatMode::DIR;
                }
            }
        }
        else {
            return -1;
        }
        let virt_nlink = VirtAddr(_st as usize + 20);
        let page_table = PageTable::from_token(token);
        if let Some(phy_nlink) = page_table.translate_va(virt_nlink) {
            let phy_nlink = phy_nlink.0 as *mut u32;
            unsafe {
                *phy_nlink = nlink;
            }
        }
        else {
            return -1;
        }
    }
    0
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let old_name = &translated_str(token, _old_name);
    let new_name = &translated_str(token, _new_name);
    if old_name == new_name {
        return -1;
    }
    use crate::fs::ROOT_INODE;
    ROOT_INODE.create_hard_link(old_name, new_name);
    0
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let name = &translated_str(token, _name);
    use crate::fs::ROOT_INODE;
    ROOT_INODE.unlink(name)
}
