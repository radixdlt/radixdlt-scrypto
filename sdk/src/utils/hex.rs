extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Encodes `data` as hex string using lowercase characters.
pub fn to_hex_string<T: AsRef<[u8]>>(data: T) -> String {
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
pub fn from_hex_string(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        Err(format!("Odd hex string length: {}", hex.len()))
    } else {
        let mut buf = Vec::<u8>::new();
        for i in (0..hex.len()).step_by(2) {
            let r = u8::from_str_radix(&hex[i..i + 2], 16);
            if r.is_err() {
                return Err(format!("Invalid hex chars: 0x{}", &hex[i..i + 2]));
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

    use crate::utils::{from_hex_string, to_hex_string};

    #[test]
    fn test_to_hex_string() {
        let input = vec![5, 15, 250];
        assert_eq!("050ffa", to_hex_string(&input));
    }

    #[test]
    fn test_from_hex_string_success() {
        assert!(from_hex_string("123").is_err());
        assert!(from_hex_string("1cxx").is_err());
    }

    #[test]
    fn test_from_hex_string_failure() {
        let input = "050ffa";
        assert_eq!(vec![5, 15, 250], from_hex_string(input).unwrap());
    }
}
