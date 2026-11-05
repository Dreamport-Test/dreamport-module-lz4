//! DreamportLz4Frame.cs: one LZ4 frame into an output of fixed size, its descriptor and every checksum it carries
//! checked, blocks no larger than it declares, and nothing after its end. A frame that would write past the output
//! is refused, never followed.

use crate::block;
use crate::xxhash32::{self, word};

const MAGIC: u32 = 0x184D2204;
const DESCRIPTOR_AT: usize = 4;
const UNCOMPRESSED: u32 = 0x8000_0000;
const REFUSED: i64 = -1;

pub fn starts_one(input: &[u8]) -> bool {
    input.len() >= 4 && word(input, 0) == MAGIC
}

/// The bytes written, which only a frame that declares no size may leave short of the output; or -1.
pub fn into(input: &[u8], output: &mut [u8]) -> i64 {
    if input.len() < 7 || word(input, 0) != MAGIC {
        return REFUSED;
    }

    let (flags, bd) = (input[4] as u32, input[5] as u32);
    if (flags >> 6) != 1 || (flags & 0x03) != 0 || (bd & 0x8F) != 0 {
        return REFUSED;
    }

    let most = block_most(bd >> 4);
    if most == 0 {
        return REFUSED;
    }

    let independent = flags & 0x20 != 0;
    let block_checks = flags & 0x10 != 0;
    let sized = flags & 0x08 != 0;
    let content_check = flags & 0x04 != 0;

    let mut at = 6usize;
    if sized {
        if input.len() < at + 9 || size(input, at) != output.len() as u64 {
            return REFUSED;
        }
        at += 8;
    }

    if input.len() <= at || input[at] != (xxhash32::of(&input[DESCRIPTOR_AT..at]) >> 8) as u8 {
        return REFUSED;
    }
    at += 1;

    let written = blocks(input, &mut at, output, most, independent, block_checks);
    if written < 0 {
        return REFUSED;
    }
    let written = written as usize;

    if content_check {
        if input.len() - at < 4 || word(input, at) != xxhash32::of(&output[..written]) {
            return REFUSED;
        }
        at += 4;
    }

    if at == input.len() {
        written as i64
    } else {
        REFUSED
    }
}

fn blocks(input: &[u8], at: &mut usize, output: &mut [u8], most: usize, independent: bool, block_checks: bool) -> i64 {
    let mut written = 0usize;
    let check = if block_checks { 4 } else { 0 };
    loop {
        if input.len() - *at < 4 {
            return REFUSED;
        }
        let w = word(input, *at);
        *at += 4;
        if w == 0 {
            return written as i64;
        }

        let stored = (w & !UNCOMPRESSED) as usize;
        if stored > most || stored as i64 > input.len() as i64 - *at as i64 - check as i64 {
            return REFUSED;
        }
        if block_checks && word(input, *at + stored) != xxhash32::of(&input[*at..*at + stored]) {
            return REFUSED;
        }

        let limit = written + most.min(output.len() - written);
        if w & UNCOMPRESSED != 0 {
            if stored > limit - written {
                return REFUSED;
            }
            output[written..written + stored].copy_from_slice(&input[*at..*at + stored]);
            written += stored;
        } else {
            let end = block::into(input, *at, stored, output, written, limit, if independent { written } else { 0 });
            if end < 0 {
                return REFUSED;
            }
            written = end as usize;
        }

        *at += stored + check;
    }
}

fn block_most(code: u32) -> usize {
    if (4..=7).contains(&code) {
        1 << (8 + 2 * code)
    } else {
        0
    }
}

fn size(b: &[u8], at: usize) -> u64 {
    word(b, at) as u64 | (word(b, at + 4) as u64) << 32
}
