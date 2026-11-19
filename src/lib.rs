//! Somnium's LZ4 decoders as a WebAssembly module (ABI v4): `lz4`, one LZ4 frame, and `lz4block`, one raw LZ4 block
//! that fills the output exactly. A translation of `dreamport/contract/Decode/DreamportLz4Frame.cs`,
//! `DreamportLz4Block.cs` and `DreamportXxHash32.cs`. It offers no reader and imports nothing.

mod block;
mod frame;
mod xxhash32;

use std::alloc::{alloc, dealloc, Layout};

const ABI_VERSION: u32 = 4;
const ALIGN: usize = 16;
const CORRUPT: i64 = -1;
const UNSUPPORTED: i64 = -2;
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
pub extern "C" fn dp_decode(name_ptr: u32, name_len: u32, in_ptr: u32, in_len: u32, out_ptr: u32, out_len: u32) -> i64 {
    if (name_ptr == 0 && name_len != 0) || (in_ptr == 0 && in_len != 0) || (out_ptr == 0 && out_len != 0) || out_len > i32::MAX as u32 {
        return BAD_ARGUMENTS;
    }
    let name: &[u8] = if name_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(name_ptr as usize as *const u8, name_len as usize) } };
    let input: &[u8] = if in_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(in_ptr as usize as *const u8, in_len as usize) } };
    let output: &mut [u8] = if out_len == 0 { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(out_ptr as usize as *mut u8, out_len as usize) } };
    let wrote = decode(name, input, output);
    if wrote < 0 {
        output.fill(0);
    }
    wrote
}

fn decode(name: &[u8], input: &[u8], output: &mut [u8]) -> i64 {
    let wrote = match name {
        b"lz4" => frame::into(input, output),
        b"lz4block" => block::whole(input, output),
        _ => return UNSUPPORTED,
    };
    if wrote == output.len() as i64 {
        wrote
    } else {
        CORRUPT
    }
}

const DESCRIPTION: &[u8] = br#"{"readers":[],"decoders":["lz4","lz4block"]}"#;

/// What this module answers, in its own words: the host routes these names to it and keeps no list of its own.
#[no_mangle]
pub extern "C" fn dp_describe() -> i64 {
    DESCRIPTION.len() as i64
}

#[no_mangle]
pub extern "C" fn dp_output() -> u32 {
    DESCRIPTION.as_ptr() as usize as u32
}
