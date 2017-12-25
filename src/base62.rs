use std::iter;
use byteorder::{BigEndian, ByteOrder};
use errors;

const BASE: u64 = 62;

const BASE62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

const UPPERCASE_OFFSET: u8 = 10;
const LOWERCASE_OFFSET: u8 = 36;


/// Calculate the actual numerical value of a base62 character.
fn base62_value(digit: &u8) -> u8 {
    if *digit >= b'0' && *digit <= b'9' {
        digit - b'0'
    } else if *digit >= b'A' && *digit <= b'Z' {
        UPPERCASE_OFFSET + (digit - b'A')
    } else {
        LOWERCASE_OFFSET + (digit - b'a')
    }
}


/// encode the given 20 byte array into a heap allocated base62 string.
/// The method used is a bit.. odd for rust. This is directly ported from the segmentio/ksuid
/// golang version which does a bunch of performance hacks. In order to avoid thinking about it
/// too much I've replicated that method wholesale.
pub fn encode(src: &[u8; 20]) -> String {
    let src_base = 4294967296;
    let dst_base = BASE;

    let mut dst: Vec<u8> = iter::repeat(b'0').take(27).collect();

    // As per the golang version, this is an O(n^2) problem, but we take N from 27 down to
    // 5 by collescing the bytes into 5 unsigned 32bit integers.
    let mut parts: [u32; 5] = [
        BigEndian::read_u32(&src[0..]),
        BigEndian::read_u32(&src[4..]),
        BigEndian::read_u32(&src[8..]),
        BigEndian::read_u32(&src[12..]),
        BigEndian::read_u32(&src[16..]),
    ];

    // This horrible C-ish code is to avoid allocating extra heap allocations to store each step in
    // the reducation. Instead we track several different offsets. I'm sorry it's come to this.
    let mut bq_index;
    let mut parts_len = 5;
    let mut n = dst.len();
    let mut remainder;
    while parts_len > 0 {
        bq_index = 0;
        remainder = 0;
        for p_index in 0..parts_len {
            let value = u64::from(parts[p_index]) + (remainder * src_base);
            let digit = value / dst_base;
            remainder = value % dst_base;
            if bq_index > 0 || digit != 0 {
                parts[bq_index] = digit as u32;
                bq_index += 1;
            }
        }
        n -= 1;
        dst[n] = BASE62_CHARS[remainder as usize];
        parts_len = bq_index;
    }
    String::from_utf8(dst).unwrap()
}

/// Decode a base64 encoded string into a vector of bytes. Once again, this is ripped wholesale
/// from segmentio/ksuid. It has the same basic structure, but reverses the encode operation.
pub fn decode(src: &str) -> Result<Vec<u8>, errors::KSUIDError> {
    let src_base = BASE;
    let dst_base = 4294967296;

    if src.len() < 27 {
        return Err(errors::KSUIDError::InvalidBase62Character{value: src.to_owned()});
    }

    let mut result: Vec<u8> = iter::repeat(0).take(20).collect();
    // I stack allocate the fool
    let mut parts: [u8;27] = [0; 27];
    let mut parts_len = 0;
    for (i, b) in src.as_bytes().iter().map(base62_value).enumerate().take(27) {
        parts[i] = b;
        parts_len += 1;
    }

    let mut bq_index;
    let mut n = result.len();
    let mut remainder;

    while parts_len > 0 {
        bq_index = 0;
        remainder = 0;

        for p_index in 0..parts_len {
            let value = u64::from(parts[p_index]) + remainder*src_base;
            let digit = value / dst_base;
            remainder = value % dst_base;
            if bq_index > 0 || digit != 0 {
                parts[bq_index] = digit as u8;
                bq_index += 1;
            }
        }
        if n < 4 {
            return Err(errors::KSUIDError::InvalidBase62Length{value: src.to_owned()});
        }

        result[n-4] = (remainder >> 24) as u8;
        result[n-3] = (remainder >> 16) as u8;
        result[n-2] = (remainder >> 8) as u8;
        result[n-1] = remainder as u8;
        n -= 4;
        parts_len = bq_index;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use test::Bencher;
    use super::*;
    use rand;
    use rand::Rng;

    #[test]
    fn b62_roundtrip() {
        let mut bytes = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut bytes);
        let encoded = encode(&bytes);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.as_slice(), &bytes);
    }

    #[bench]
    fn bench_b62_encode(b: &mut Bencher) {
        let mut bytes = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut bytes);
        b.iter(|| encode(&bytes));
    }
    #[bench]
    fn bench_b62_decode(b: &mut Bencher) {
        let mut bytes = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut bytes);
        let encoded = encode(&bytes);
        b.iter(|| decode(&encoded));
    }
}
