//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    // Check if ts is null
    if ts.is_null() {
        return -1;
    }

    // Use translated_byte_buffer to handle the case where TimeVal might be split across pages
    use crate::mm::translated_byte_buffer;
    use crate::task::current_user_token;
    use crate::timer::get_time_us;

    let token = current_user_token();
    let buffers = translated_byte_buffer(token, ts as *const u8, core::mem::size_of::<TimeVal>());

    if buffers.is_empty() {
        return -1;
    }

    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    // Copy the TimeVal data to user space using the translated buffers
    let time_val_bytes = unsafe {
        core::slice::from_raw_parts(
            &time_val as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };

    let mut offset = 0;
    for buffer in buffers {
        let copy_len = buffer.len().min(time_val_bytes.len() - offset);
        buffer[..copy_len].copy_from_slice(&time_val_bytes[offset..offset + copy_len]);
        offset += copy_len;
        if offset >= time_val_bytes.len() {
            break;
        }
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");

    use crate::task::{translate_user_ptr, translate_user_ptr_readonly};

    match trace_request {
        0 => {
            // Read operation
            let ptr = id as *const u8;
            if ptr == isize::MAX as *const u8 {
                return -1;
            }
            if let Some(byte_ref) = translate_user_ptr_readonly(ptr) {
                *byte_ref as isize
            } else {
                -1
            }
        }
        1 => {
            // Write operation
            let ptr = id as *mut u8;
            if ptr == isize::MAX as *mut u8 {
                return -1;
            }
            if let Some(byte_ref) = translate_user_ptr(ptr) {
                *byte_ref = data as u8;
                0
            } else {
                -1
            }
        }
        2 => {
            // Syscall count - return the count for syscall with id
            use crate::task::get_current_syscall_count;
            get_current_syscall_count(id) as isize
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!(
        "kernel: sys_mmap start={:#x}, len={}, prot={}",
        start,
        len,
        prot
    );
    use crate::task::current_mmap;
    current_mmap(start, len, prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap start={:#x}, len={}", start, len);
    use crate::task::current_munmap;
    current_munmap(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
