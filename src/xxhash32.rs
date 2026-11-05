//! DreamportXxHash32.cs: xxHash32 with seed 0, the checksum an LZ4 frame keeps of its descriptor, its blocks and its content.

const PRIME1: u32 = 2654435761;
const PRIME2: u32 = 2246822519;
const PRIME3: u32 = 3266489917;
const PRIME4: u32 = 668265263;
const PRIME5: u32 = 374761393;

pub fn of(data: &[u8]) -> u32 {
    let count = data.len();
    let mut at = 0usize;
    let mut hash: u32;

    if count >= 16 {
        let (mut v1, mut v2, mut v3, mut v4) = (PRIME1.wrapping_add(PRIME2), PRIME2, 0u32, 0u32.wrapping_sub(PRIME1));
        while at + 16 <= count {
            v1 = round(v1, word(data, at));
            v2 = round(v2, word(data, at + 4));
            v3 = round(v3, word(data, at + 8));
            v4 = round(v4, word(data, at + 12));
            at += 16;
        }
        hash = v1.rotate_left(1).wrapping_add(v2.rotate_left(7)).wrapping_add(v3.rotate_left(12)).wrapping_add(v4.rotate_left(18));
    } else {
        hash = PRIME5;
    }

    hash = hash.wrapping_add(count as u32);
    while at + 4 <= count {
        hash = hash.wrapping_add(word(data, at).wrapping_mul(PRIME3)).rotate_left(17).wrapping_mul(PRIME4);
        at += 4;
    }
    while at < count {
        hash = hash.wrapping_add((data[at] as u32).wrapping_mul(PRIME5)).rotate_left(11).wrapping_mul(PRIME1);
        at += 1;
    }

    hash ^= hash >> 15;
    hash = hash.wrapping_mul(PRIME2);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(PRIME3);
    hash ^= hash >> 16;
    hash
}

pub fn word(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn round(lane: u32, word: u32) -> u32 {
    lane.wrapping_add(word.wrapping_mul(PRIME2)).rotate_left(13).wrapping_mul(PRIME1)
}
