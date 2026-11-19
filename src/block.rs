//! DreamportLz4Block.cs: one LZ4 block written into the output from a position, never past a limit, its matches
//! reaching back no further than allowed.

const MIN_MATCH: usize = 4;
const REFUSED: i64 = -1;

/// One block that fills the output exactly, as a container that keeps each block's two sizes stores it.
pub fn whole(input: &[u8], output: &mut [u8]) -> i64 {
    if input.is_empty() {
        return REFUSED;
    }
    let end = into(input, 0, input.len(), output, 0, output.len(), 0);
    if end == output.len() as i64 {
        end
    } else {
        REFUSED
    }
}

/// Where the output ends after the block, or -1 when the block is not one this output can hold.
pub fn into(input: &[u8], at: usize, count: usize, output: &mut [u8], written: usize, limit: usize, reach_from: usize) -> i64 {
    let (mut i, end, mut o) = (at, at + count, written);

    while i < end {
        let token = input[i] as usize;
        i += 1;

        let mut literals = token >> 4;
        if literals == 15 && !longer(input, &mut i, end, &mut literals) {
            return REFUSED;
        }
        if literals > end - i || literals > limit - o {
            return REFUSED;
        }

        output[o..o + literals].copy_from_slice(&input[i..i + literals]);
        i += literals;
        o += literals;
        if i == end {
            return o as i64;
        }

        if end - i < 2 {
            return REFUSED;
        }
        let offset = input[i] as usize | (input[i + 1] as usize) << 8;
        i += 2;
        if offset == 0 || offset > o - reach_from {
            return REFUSED;
        }

        let mut length = token & 15;
        if length == 15 && !longer(input, &mut i, end, &mut length) {
            return REFUSED;
        }
        length += MIN_MATCH;
        if length > limit - o {
            return REFUSED;
        }

        copy(output, o - offset, o, length);
        o += length;
    }

    REFUSED
}

/// A match may overlap what it writes: then each byte is copied after the one it repeats.
fn copy(output: &mut [u8], from: usize, to: usize, count: usize) {
    if to - from >= count {
        output.copy_within(from..from + count, to);
        return;
    }
    for k in 0..count {
        output[to + k] = output[from + k];
    }
}

fn longer(input: &[u8], i: &mut usize, end: usize, length: &mut usize) -> bool {
    loop {
        if *i >= end || *length > i32::MAX as usize - 255 {
            return false;
        }
        let more = input[*i] as usize;
        *i += 1;
        *length += more;
        if more != 255 {
            return true;
        }
    }
}
