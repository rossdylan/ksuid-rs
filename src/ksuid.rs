use base62;
use byteorder::{BigEndian, ByteOrder};
use chrono::prelude::Utc;
use chrono::{DateTime, NaiveDateTime};
use errors;
use rand;
use rand::Rng;
use std::fmt;


// Define ksuid constants
const EPOCH_START: i64 = 1400000000;
const TIMESTAMP_LENGTH: usize = 4;
const PAYLOAD_LENGTH: usize = 16;
const BYTE_LENGTH: usize = TIMESTAMP_LENGTH + PAYLOAD_LENGTH;

// Length of the base62 encoded string version
const ENCODED_LENGTH: u64 = 27;

// A string-encoded maximum value for a KSUID
const MAX_STRING_ENCODED: &str  = "aWgEPTl1tmebfsQzFP4bxwgy80V";

#[derive(Debug, Default, PartialEq)]
pub struct KSUID(pub [u8; BYTE_LENGTH]);


fn to_ksuid_time(t: DateTime<Utc>) -> u32 {
    (t.timestamp() - EPOCH_START) as u32
}

fn from_ksuid_time(t: u32) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(
            i64::from(t) + EPOCH_START,
            0,
        ),
        Utc,
    )
}

impl fmt::Display for KSUID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_base62())
    }
}

impl KSUID {

    /// Create a new random `KSUID` based on the current time and some random data
    /// # Example
    /// ```
    /// use ksuid::KSUID;
    ///
    /// let uid = KSUID::new();
    /// let other = KSUID::new();
    /// assert_ne!(uid, other);
    /// ```
    pub fn new() -> Self {
        let time = to_ksuid_time(Utc::now());
        let mut bytes = [0u8; BYTE_LENGTH];
        rand::thread_rng().fill_bytes(&mut bytes);
        BigEndian::write_u32(&mut bytes, time);
        KSUID(bytes)
    }

    /// Create a new `KSUID` from it's raw components.
    /// # Example
    /// ```
    /// use ksuid::KSUID;
    ///
    /// let uid = KSUID::new();
    /// let other = KSUID::from_parts(uid.timestamp(), uid.payload()).unwrap();
    /// assert_eq!(other, uid)
    /// ```
    pub fn from_parts(ts: DateTime<Utc>, payload: &[u8]) -> Result<Self, errors::KSUIDError> {
        if payload.len() < PAYLOAD_LENGTH {
            return Err(errors::KSUIDError::SliceTooSmall{length: payload.len()})
        }
        let mut bytes = [0u8; BYTE_LENGTH];
        BigEndian::write_u32(&mut bytes, to_ksuid_time(ts));
        bytes[TIMESTAMP_LENGTH..].clone_from_slice(&payload[..PAYLOAD_LENGTH]);
        Ok(KSUID(bytes))
    }

    /// Return a ksuid built from a byte slice. The slice could be of arbitary size. The first 20
    /// bytes will be the only ones used. If the slice is too small an error is returned.
    /// # Example
    /// ```
    /// use ksuid::KSUID;
    ///
    /// let uid = KSUID::new();
    /// let other = KSUID::from_bytes(uid.as_bytes()).unwrap();
    /// assert_eq!(other, uid);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, errors::KSUIDError> {
        if bytes.len() < BYTE_LENGTH {
            return Err(errors::KSUIDError::SliceTooSmall{length: bytes.len()})
        }
        let mut arr = [0u8; BYTE_LENGTH];
        (&mut arr).copy_from_slice(&bytes[..BYTE_LENGTH]);
        Ok(KSUID(arr))
    }

    pub fn from_base62(string: &str) -> Result<Self, errors::KSUIDError> {
        base62::decode(string).and_then(|bytes| {
            Self::from_bytes(bytes.as_slice())
        })
    }


    /// Return the timestamp portion of a ksuid as a `time::Timespec` struct
    pub fn timestamp(&self) -> DateTime<Utc> {
        from_ksuid_time(BigEndian::read_u32(&self.0))
    }

    /// Return the random payload portion of the ksuid as a reference to the underlying array
    pub fn payload(&self) -> &[u8] {
        &(&self.0)[TIMESTAMP_LENGTH..]
    }

    /// Encode the underlying bytes as a base62 `String`
    pub fn to_base62(&self) -> String {
        base62::encode(&self.0)
    }

    /// Return a reference to the bytes that make up a ksuid.
    pub fn as_bytes(&self) -> &[u8] {
        &(self.0)
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;
    use super::*;
    use std::iter;

    #[test]
    fn test_ksuid_base62() {
        let zero = KSUID::from_bytes(&[0; 20]).unwrap();
        let expected = String::from_utf8(
            iter::repeat('0' as u8).take(ENCODED_LENGTH as usize).collect()
        ).unwrap(); 
        assert_eq!(zero.to_base62(), expected);

        let uid = KSUID::new();
        let other = KSUID::from_base62(&uid.to_base62()).unwrap();
        println!("ksuid: {}", other);
        assert_eq!(uid, other);
    }
    #[test]
    fn invalid_from_bytes() {
        let failed = match KSUID::from_bytes(&[0;2]) {
            Err(_) => true,
            Ok(_) => false,
        };
        assert!(failed);
    }

    #[test]
    fn test_parse_golang() {
        let res = KSUID::from_base62(&"0yEaNH85uGuB4bz7EoWhX228k65");
        assert!(res.is_ok());
        let uid = res.unwrap();
        println!("timestamp: {}, payload: {:?}", uid.timestamp(), uid.payload());
    }

    #[bench]
    fn bench_ksuid_new(b: &mut Bencher) {
        b.iter(|| KSUID::new());
    }

}
