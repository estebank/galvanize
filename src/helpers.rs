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
        // Truncate to 32 bits and remove sign.
        h = (((h << 5) + h) ^ x) & Wrapping(0xffffffff);
    }
    h.0
}

/// Represent an iterable of bytes as "lossy" `utf8` `String`.
///
/// If the byte cannot be represented as an `utf8` character, it'll be replaced
/// with a `?`.
pub fn vec2str(v: &[u8]) -> String {
    String::from_utf8_lossy(v).into_owned()
}
