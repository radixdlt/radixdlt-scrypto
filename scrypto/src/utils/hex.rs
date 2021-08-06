extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
pub enum DecodeHexError {
    InvalidCharacter,
    InvalidLength,
}

/// Encodes `data` as hex string using lowercase characters.
pub fn hex_encode<T: AsRef<[u8]>>(data: T) -> String {
    let mut buf = String::new();

    for b in data.as_ref() {
        fn hex_from_digit(num: u8) -> char {
            if num < 10 {
                (b'0' + num) as char
            } else {
                (b'a' + num - 10) as char
            }
        }
        buf.push(hex_from_digit(b / 16));
        buf.push(hex_from_digit(b % 16));
    }

    buf
}

/// Decode a hex string into a byte vector.
pub fn hex_decode(hex: &str) -> Result<Vec<u8>, DecodeHexError> {
    if hex.len() % 2 != 0 {
        Err(DecodeHexError::InvalidLength)
    } else {
        let mut buf = Vec::<u8>::new();
        for i in (0..hex.len()).step_by(2) {
            let r = u8::from_str_radix(&hex[i..i + 2], 16);
            if r.is_err() {
                return Err(DecodeHexError::InvalidCharacter);
            } else {
                buf.push(r.unwrap());
            }
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec;

    use crate::utils::{hex_decode, hex_encode};

    #[test]
    fn test_hex_encode() {
        let input = vec![5, 15, 250];
        assert_eq!("050ffa", hex_encode(&input));
    }

    #[test]
    fn test_hex_decode_success() {
        assert!(hex_decode("123").is_err());
        assert!(hex_decode("1cxx").is_err());
    }

    #[test]
    fn test_hex_decode_failure() {
        let input = "050ffa";
        assert_eq!(vec![5, 15, 250], hex_decode(input).unwrap());
    }
}
