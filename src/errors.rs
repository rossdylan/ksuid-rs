#[derive(Debug, Fail)]
pub enum KSUIDError {
    #[fail(display = "byte slice too small: {}", length)]
    SliceTooSmall {
        length: usize,
    },
    #[fail(display = "invalid character in base62 string: '{}'", value)]
    InvalidBase62Character {
        value: String,
    },
    #[fail(display = "base62 string has invalid length: '{}'", value)]
    InvalidBase62Length {
        value: String,
    },
}
