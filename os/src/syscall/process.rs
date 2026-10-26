//! Process management syscalls
use crate::config::PAGE_SIZE;
use crate::mm::{translated_byte_buffer, MapPermission, PageTable, PTEFlags, VirtAddr};
use crate::task::{
    change_program_brk, current_memory_set_mmap, current_memory_set_munmap, current_user_token,
    exit_current_and_run_next, get_syscall_count, suspend_current_and_run_next,
};
use crate::timer::get_time_us;


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
    let us = get_time_us();
    let tv = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    // ts is a user virtual address, may cross pages, so write through page table
    let buffers = translated_byte_buffer(
        current_user_token(),
        ts as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    unsafe {
        let tv_bytes: [u8; 16] = core::mem::transmute(tv);
        let mut offset = 0;
        for buf in buffers {
            let n = buf.len();
            buf.copy_from_slice(&tv_bytes[offset..offset + n]);
            offset += n;
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
const TRACE_READ: usize = 0;
const TRACE_WRITE: usize = 1;
const TRACE_SYSCALL: usize = 2;
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        TRACE_READ => {
            let page_table = PageTable::from_token(current_user_token());
            let va = VirtAddr::from(id);
            if let Some(pte) = page_table.translate(va.floor()) {
                let flags = pte.flags();
                if flags.contains(PTEFlags::U) && pte.readable() {
                    return pte.ppn().get_bytes_array()[va.page_offset()] as isize;
                }
            }
            -1
        }
        TRACE_WRITE => {
            let page_table = PageTable::from_token(current_user_token());
            let va = VirtAddr::from(id);
            if let Some(pte) = page_table.translate(va.floor()) {
                let flags = pte.flags();
                if flags.contains(PTEFlags::U) && pte.writable() {
                    pte.ppn().get_bytes_array()[va.page_offset()] = data as u8;
                    return 0;
                }
            }
            -1
        }
        TRACE_SYSCALL => get_syscall_count(id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    // prot must be R/W/R|W only (bits 0-1), and not zero
    if prot == 0 || prot & !3 != 0 {
        return -1;
    }
    // start must be page-aligned
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let mut perm = MapPermission::U;
    if prot & 1 != 0 {
        perm |= MapPermission::R;
    }
    if prot & 2 != 0 {
        perm |= MapPermission::W;
    }
    let ok = current_memory_set_mmap(
        VirtAddr::from(start),
        VirtAddr::from(start + len),
        perm,
    );
    if ok {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let ok = current_memory_set_munmap(
        VirtAddr::from(start),
        VirtAddr::from(start + len),
    );
    if ok {
        0
    } else {
        -1
    }
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
