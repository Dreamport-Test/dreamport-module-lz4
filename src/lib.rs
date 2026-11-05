//! Somnium's LZ4 decoders as a WebAssembly module (ABI v2): `lz4`, one LZ4 frame, and `lz4block`, one raw LZ4 block
//! that fills the output exactly. A translation of `dreamport/contract/Decode/DreamportLz4Frame.cs`,
//! `DreamportLz4Block.cs` and `DreamportXxHash32.cs`. It offers no reader and imports nothing.

mod block;
mod frame;
mod xxhash32;

use std::alloc::{alloc, dealloc, Layout};

const ABI_VERSION: u32 = 2;
const ALIGN: usize = 16;
const CORRUPT: i64 = -1;
const BAD_ARGUMENTS: i64 = -5;

#[no_mangle]
pub extern "C" fn dp_abi() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub extern "C" fn dp_alloc(len: u32) -> u32 {
    if len == 0 {
        return 0;
    }
    match Layout::from_size_align(len as usize, ALIGN) {
        Ok(layout) => unsafe { alloc(layout) as usize as u32 },
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn dp_free(ptr: u32, len: u32) {
    if ptr == 0 || len == 0 {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len as usize, ALIGN) {
        unsafe { dealloc(ptr as usize as *mut u8, layout) }
    }
}

#[no_mangle]
pub extern "C" fn dp_decode(in_ptr: u32, in_len: u32, out_ptr: u32, out_len: u32) -> i64 {
    if (in_ptr == 0 && in_len != 0) || (out_ptr == 0 && out_len != 0) || out_len > i32::MAX as u32 {
        return BAD_ARGUMENTS;
    }
    let input: &[u8] = if in_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(in_ptr as usize as *const u8, in_len as usize) } };
    let output: &mut [u8] = if out_len == 0 { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(out_ptr as usize as *mut u8, out_len as usize) } };
    let wrote = decode(input, output);
    if wrote < 0 {
        output.fill(0);
    }
    wrote
}

/// ABI v2 gives `dp_decode` no decoder name, so the input says which: a frame starts with the LZ4 magic, and a whole
/// raw block never can (its first token would copy a match from before the output's start).
fn decode(input: &[u8], output: &mut [u8]) -> i64 {
    let wrote = if frame::starts_one(input) { frame::into(input, output) } else { block::whole(input, output) };
    if wrote == output.len() as i64 {
        wrote
    } else {
        CORRUPT
    }
}
