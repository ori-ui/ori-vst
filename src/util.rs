use std::{ffi::c_char, mem, ptr};

pub unsafe fn strcpy(src: &str, dst: &mut [c_char]) {
    if src.len() >= dst.len() {
        return;
    }

    ptr::copy_nonoverlapping(src.as_ptr() as *const c_char, dst.as_mut_ptr(), src.len());
}

pub unsafe fn u16strcpy(src: &str, dst: &mut [i16]) {
    if dst.is_empty() {
        return;
    }

    for (i, c) in src.encode_utf16().enumerate() {
        if i >= dst.len() {
            break;
        }

        // SAFETY: `i16` has no invalid bit patterns.
        dst[i] = mem::transmute(c);
    }
}
