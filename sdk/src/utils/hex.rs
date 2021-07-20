extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut buf = String::new();

    for b in bytes {
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

pub fn decode_hex(s: &str) -> Result<Vec<u8>, &str> {
    if s.len() % 2 != 0 {
        Err("Odd hex string length")
    } else {
        let mut buf = Vec::<u8>::new();
        for i in (0..s.len()).step_by(2) {
            let r = u8::from_str_radix(&s[i..i + 2], 16);
            if r.is_err() {
                return Err("Invalid hex char");
            } else {
                buf.push(r.unwrap());
            }
        }
        Ok(buf)
    }
}
