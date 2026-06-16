use core::ffi::{c_int, c_void};

/// C-ABI shim for `memchr` required by mbedtls C source code.
/// This fulfills the linker's requirement for the missing libc symbol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memchr(s: *const c_void, c: c_int, n: usize) -> *mut c_void {
    let p = s as *const u8;
    let target = c as u8;

    for i in 0..n {
        if *p.add(i) == target {
            return p.add(i) as *mut c_void;
        }
    }

    core::ptr::null_mut()
}
