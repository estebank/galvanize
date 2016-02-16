use std::num::Wrapping;
use std::string::String;


/// DJB hash function
pub fn hash(string: &[u8]) -> u32 {
    let mut h: Wrapping<u32> = Wrapping(5381);
    for c in string.iter() {
        let x: Wrapping<u32> = Wrapping(c.to_owned() as u32);
        h = (((h << 5) + h) ^ x) & Wrapping(0xffffffff);
    }
    h.0
}

#[inline]
pub fn pack(v: u32) -> [u8; 4] {
    [v as u8, (v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8]
}

#[inline]
pub fn unpack(v: [u8; 4]) -> u32 {
    ((v[0] as u32) | ((v[1] as u32) << 8) | ((v[2] as u32) << 16) | ((v[3] as u32) << 24))
}

pub fn vec2str<'a>(v: &'a [u8]) -> String {
    String::from_utf8_lossy(&v[..]).into_owned()
}
