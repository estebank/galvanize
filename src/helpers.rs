//! Various functions that are used across both the writer and reader modules.
//!
//! You shouldn't need to use this module directly.
use std::num::Wrapping;


/// DJB hash function
///
/// It is `h = ((h << 5) + h) ^ c`, with a starting hash of `5381`.
pub fn hash(string: &[u8]) -> u32 {
    let mut h: Wrapping<u32> = Wrapping(5381);
    for c in string.iter() {
        let x: Wrapping<u32> = Wrapping(c.to_owned() as u32);
        h = (((h << 5) + h) ^ x) & Wrapping(0xffffffff);
    }
    h.0
}

/// Get array of bytes from an `u32`.
#[inline]
pub fn pack(v: u32) -> [u8; 4] {
    [v as u8, (v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8]
}

/// Get an `u32` from an array of 4 bytes.
#[inline]
pub fn unpack(v: [u8; 4]) -> u32 {
    ((v[0] as u32) | ((v[1] as u32) << 8) | ((v[2] as u32) << 16) | ((v[3] as u32) << 24))
}

/// Represent an iterable of bytes as "lossy" `utf8` `String`.
///
/// If the byte cannot be represented as an `utf8` character, it'll be replaced
/// with a `?`.
pub fn vec2str<'a>(v: &'a [u8]) -> String {
    String::from_utf8_lossy(&v[..]).into_owned()
}
