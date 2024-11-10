// LICENSING NOTICE:
// This file contains code sourced from the Rust core.
//
// This original source is dual licensed under:
// - Apache License, Version 2.0
// - MIT license
//
// The code in this file has been implemented by Radix® pursuant to an Apache 2 licence and has
// been modified by Radix® and is now licensed pursuant to the Radix® Open-Source Licence.
//
// Each sourced code fragment includes an inline attribution to the original source file in a
// comment starting "SOURCE: ..."
//
// Modifications from the original source are captured in two places:
// * Initial changes to get the code functional/integrated are marked by inline "INITIAL-MODIFICATION: ..." comments
// * Subsequent changes to the code are captured in the git commit history

use crate::prelude::*;

// INTERNAL COMMENT:
// We use code taken from Rust core instead of code from a unicode library
// to align with their Debug intepretation, and to avoid introducing a large
// dependency which might slow down compilation or allow an attack.

/// Efficiently escapes a string, using the given escape behaviour
/// and escape formatting.
pub trait CustomCharEscaper: Sized {
    /// For efficiency, all ASCII characters in the range `0x20 <= b <= 0x7E`
    /// except `"` and `\` are considered as not-needing escaping, and the
    /// `resolve_escape_behaviour` function is not called for them.
    fn resolve_escape_behaviour(c: char) -> EscapeBehaviour;

    fn format_unicode_escaped_char(f: &mut impl fmt::Write, c: char) -> fmt::Result;

    fn format_string_start(f: &mut impl fmt::Write) -> fmt::Result {
        f.write_str("\"")
    }

    fn format_string_end(f: &mut impl fmt::Write) -> fmt::Result {
        f.write_str("\"")
    }

    fn escaped<'a>(input: &'a str) -> CustomEscaped<'a, Self> {
        CustomEscaped::new(input)
    }
}

pub struct CustomEscaped<'a, E: CustomCharEscaper> {
    input: &'a str,
    escaper: PhantomData<E>,
}

impl<'a, E: CustomCharEscaper> CustomEscaped<'a, E> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            escaper: PhantomData,
        }
    }
}

impl<E: CustomCharEscaper> fmt::Display for CustomEscaped<'_, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        format_custom_escaped::<E>(f, self.input)
    }
}

pub enum EscapeBehaviour {
    None,
    Replace(&'static str),
    UnicodeEscape,
}

// SOURCE: https://github.com/rust-lang/rust/blob/1.81.0/library/core/src/fmt/mod.rs
// INITIAL-MODIFICATION:
// - Moved the wrapping quotes, and inner escaping logic to a
//   `CustomCharEscaper` trait

/// Inspired by `impl Debug for str` in std, which tries to efficiently copy
/// ranges of safe-to-encode characters.
pub fn format_custom_escaped<E: CustomCharEscaper>(
    f: &mut impl fmt::Write,
    input: &str,
) -> core::fmt::Result {
    let mut printable_range = 0..0;

    fn could_need_escaping(b: u8) -> bool {
        b > 0x7E || b < 0x20 || b == b'\\' || b == b'"'
    }

    E::format_string_start(f)?;

    // the loop here first skips over runs of printable ASCII as a fast path.
    // other chars (unicode, or ASCII that needs escaping) are then handled per-`char`.
    let mut rest = input;
    while rest.len() > 0 {
        let Some(non_printable_start) =
            rest.as_bytes().iter().position(|&b| could_need_escaping(b))
        else {
            printable_range.end += rest.len();
            break;
        };

        printable_range.end += non_printable_start;
        // SAFETY: the position was derived from an iterator, so is known to be within bounds, and at a char boundary
        rest = unsafe { rest.get_unchecked(non_printable_start..) };

        let mut chars = rest.chars();
        if let Some(c) = chars.next() {
            match E::resolve_escape_behaviour(c) {
                EscapeBehaviour::None => {}
                EscapeBehaviour::Replace(replacement) => {
                    f.write_str(&input[printable_range.clone()])?;
                    f.write_str(replacement)?;
                    printable_range.start = printable_range.end + c.len_utf8();
                }
                EscapeBehaviour::UnicodeEscape => {
                    f.write_str(&input[printable_range.clone()])?;
                    E::format_unicode_escaped_char(f, c)?;
                    printable_range.start = printable_range.end + c.len_utf8();
                }
            }
            printable_range.end += c.len_utf8();
        }
        rest = chars.as_str();
    }

    f.write_str(&input[printable_range])?;
    E::format_string_end(f)?;
    Ok(())
}

// Logic comes from re-interpreting `escape_debug_ext` in
// https://github.com/rust-lang/rust/blob/1.81.0/library/core/src/char/methods.rs
pub fn rust_1_81_should_unicode_escape_in_debug_str(char: char) -> bool {
    rust_core_1_81_grapheme_extend::lookup(char) || !rust_core_1_81_is_printable::is_printable(char)
}

/// As per the JSON spec, we escape unicode in terms of its UTF-16 encoding,
/// in one or two `\uXXXX` characters.
pub fn format_json_utf16_escaped_char(f: &mut impl fmt::Write, c: char) -> fmt::Result {
    match c.len_utf16() {
        1 => {
            let mut encoded = [0u16; 1];
            c.encode_utf16(&mut encoded);
            encode_single_utf16(f, encoded[0])?;
        }
        2 => {
            let mut encoded = [0u16; 2];
            c.encode_utf16(&mut encoded);
            encode_single_utf16(f, encoded[0])?;
            encode_single_utf16(f, encoded[1])?;
        }
        // SAFETY: char::len_utf16() is guaranteed to return 1 or 2
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
    Ok(())
}

const LOWER_CASE_HEX_CARS: &[u8; 16] = b"0123456789abcdef";

fn encode_single_utf16(f: &mut impl core::fmt::Write, value: u16) -> core::fmt::Result {
    let [upper_byte, lower_byte] = value.to_be_bytes();
    let output = [
        LOWER_CASE_HEX_CARS[((upper_byte & 0xf0) >> 4) as usize],
        LOWER_CASE_HEX_CARS[(upper_byte & 0x0f) as usize],
        LOWER_CASE_HEX_CARS[((lower_byte & 0xf0) >> 4) as usize],
        LOWER_CASE_HEX_CARS[(lower_byte & 0x0f) as usize],
    ];
    write!(
        f,
        "\\u{}",
        // SAFETY: all chars are ASCII
        unsafe { core::str::from_utf8_unchecked(&output) }
    )
}

// SOURCE: https://github.com/rust-lang/rust/blob/1.81.0/library/core/src/unicode/printable.rs
// INITIAL-MODIFICATION: Re-arrange methods and put into a mod.
mod rust_core_1_81_is_printable {
    // NOTE: The following code was generated by "library/core/src/unicode/printable.py",
    //       do not edit directly!
    fn check(x: u16, singletonuppers: &[(u8, u8)], singletonlowers: &[u8], normal: &[u8]) -> bool {
        let xupper = (x >> 8) as u8;
        let mut lowerstart = 0;
        for &(upper, lowercount) in singletonuppers {
            let lowerend = lowerstart + lowercount as usize;
            if xupper == upper {
                for &lower in &singletonlowers[lowerstart..lowerend] {
                    if lower == x as u8 {
                        return false;
                    }
                }
            } else if xupper < upper {
                break;
            }
            lowerstart = lowerend;
        }

        let mut x = x as i32;
        let mut normal = normal.iter().cloned();
        let mut current = true;
        while let Some(v) = normal.next() {
            let len = if v & 0x80 != 0 {
                ((v & 0x7f) as i32) << 8 | normal.next().unwrap() as i32
            } else {
                v as i32
            };
            x -= len;
            if x < 0 {
                break;
            }
            current = !current;
        }
        current
    }

    pub(crate) fn is_printable(x: char) -> bool {
        let x = x as u32;
        let lower = x as u16;

        if x < 32 {
            // ASCII fast path
            false
        } else if x < 127 {
            // ASCII fast path
            true
        } else if x < 0x10000 {
            check(lower, SINGLETONS0U, SINGLETONS0L, NORMAL0)
        } else if x < 0x20000 {
            check(lower, SINGLETONS1U, SINGLETONS1L, NORMAL1)
        } else {
            if 0x2a6e0 <= x && x < 0x2a700 {
                return false;
            }
            if 0x2b73a <= x && x < 0x2b740 {
                return false;
            }
            if 0x2b81e <= x && x < 0x2b820 {
                return false;
            }
            if 0x2cea2 <= x && x < 0x2ceb0 {
                return false;
            }
            if 0x2ebe1 <= x && x < 0x2ebf0 {
                return false;
            }
            if 0x2ee5e <= x && x < 0x2f800 {
                return false;
            }
            if 0x2fa1e <= x && x < 0x30000 {
                return false;
            }
            if 0x3134b <= x && x < 0x31350 {
                return false;
            }
            if 0x323b0 <= x && x < 0xe0100 {
                return false;
            }
            if 0xe01f0 <= x && x < 0x110000 {
                return false;
            }
            true
        }
    }

    #[rustfmt::skip]
    const SINGLETONS0U: &[(u8, u8)] = &[
        (0x00, 1),
        (0x03, 5),
        (0x05, 6),
        (0x06, 2),
        (0x07, 6),
        (0x08, 7),
        (0x09, 17),
        (0x0a, 28),
        (0x0b, 25),
        (0x0c, 26),
        (0x0d, 16),
        (0x0e, 12),
        (0x0f, 4),
        (0x10, 3),
        (0x12, 18),
        (0x13, 9),
        (0x16, 1),
        (0x17, 4),
        (0x18, 1),
        (0x19, 3),
        (0x1a, 7),
        (0x1b, 1),
        (0x1c, 2),
        (0x1f, 22),
        (0x20, 3),
        (0x2b, 3),
        (0x2d, 11),
        (0x2e, 1),
        (0x30, 4),
        (0x31, 2),
        (0x32, 1),
        (0xa7, 2),
        (0xa9, 2),
        (0xaa, 4),
        (0xab, 8),
        (0xfa, 2),
        (0xfb, 5),
        (0xfd, 2),
        (0xfe, 3),
        (0xff, 9),
    ];
    #[rustfmt::skip]
    const SINGLETONS0L: &[u8] = &[
        0xad, 0x78, 0x79, 0x8b, 0x8d, 0xa2, 0x30, 0x57,
        0x58, 0x8b, 0x8c, 0x90, 0x1c, 0xdd, 0x0e, 0x0f,
        0x4b, 0x4c, 0xfb, 0xfc, 0x2e, 0x2f, 0x3f, 0x5c,
        0x5d, 0x5f, 0xe2, 0x84, 0x8d, 0x8e, 0x91, 0x92,
        0xa9, 0xb1, 0xba, 0xbb, 0xc5, 0xc6, 0xc9, 0xca,
        0xde, 0xe4, 0xe5, 0xff, 0x00, 0x04, 0x11, 0x12,
        0x29, 0x31, 0x34, 0x37, 0x3a, 0x3b, 0x3d, 0x49,
        0x4a, 0x5d, 0x84, 0x8e, 0x92, 0xa9, 0xb1, 0xb4,
        0xba, 0xbb, 0xc6, 0xca, 0xce, 0xcf, 0xe4, 0xe5,
        0x00, 0x04, 0x0d, 0x0e, 0x11, 0x12, 0x29, 0x31,
        0x34, 0x3a, 0x3b, 0x45, 0x46, 0x49, 0x4a, 0x5e,
        0x64, 0x65, 0x84, 0x91, 0x9b, 0x9d, 0xc9, 0xce,
        0xcf, 0x0d, 0x11, 0x29, 0x3a, 0x3b, 0x45, 0x49,
        0x57, 0x5b, 0x5c, 0x5e, 0x5f, 0x64, 0x65, 0x8d,
        0x91, 0xa9, 0xb4, 0xba, 0xbb, 0xc5, 0xc9, 0xdf,
        0xe4, 0xe5, 0xf0, 0x0d, 0x11, 0x45, 0x49, 0x64,
        0x65, 0x80, 0x84, 0xb2, 0xbc, 0xbe, 0xbf, 0xd5,
        0xd7, 0xf0, 0xf1, 0x83, 0x85, 0x8b, 0xa4, 0xa6,
        0xbe, 0xbf, 0xc5, 0xc7, 0xcf, 0xda, 0xdb, 0x48,
        0x98, 0xbd, 0xcd, 0xc6, 0xce, 0xcf, 0x49, 0x4e,
        0x4f, 0x57, 0x59, 0x5e, 0x5f, 0x89, 0x8e, 0x8f,
        0xb1, 0xb6, 0xb7, 0xbf, 0xc1, 0xc6, 0xc7, 0xd7,
        0x11, 0x16, 0x17, 0x5b, 0x5c, 0xf6, 0xf7, 0xfe,
        0xff, 0x80, 0x6d, 0x71, 0xde, 0xdf, 0x0e, 0x1f,
        0x6e, 0x6f, 0x1c, 0x1d, 0x5f, 0x7d, 0x7e, 0xae,
        0xaf, 0x7f, 0xbb, 0xbc, 0x16, 0x17, 0x1e, 0x1f,
        0x46, 0x47, 0x4e, 0x4f, 0x58, 0x5a, 0x5c, 0x5e,
        0x7e, 0x7f, 0xb5, 0xc5, 0xd4, 0xd5, 0xdc, 0xf0,
        0xf1, 0xf5, 0x72, 0x73, 0x8f, 0x74, 0x75, 0x96,
        0x26, 0x2e, 0x2f, 0xa7, 0xaf, 0xb7, 0xbf, 0xc7,
        0xcf, 0xd7, 0xdf, 0x9a, 0x00, 0x40, 0x97, 0x98,
        0x30, 0x8f, 0x1f, 0xd2, 0xd4, 0xce, 0xff, 0x4e,
        0x4f, 0x5a, 0x5b, 0x07, 0x08, 0x0f, 0x10, 0x27,
        0x2f, 0xee, 0xef, 0x6e, 0x6f, 0x37, 0x3d, 0x3f,
        0x42, 0x45, 0x90, 0x91, 0x53, 0x67, 0x75, 0xc8,
        0xc9, 0xd0, 0xd1, 0xd8, 0xd9, 0xe7, 0xfe, 0xff,
    ];
    #[rustfmt::skip]
    const SINGLETONS1U: &[(u8, u8)] = &[
        (0x00, 6),
        (0x01, 1),
        (0x03, 1),
        (0x04, 2),
        (0x05, 7),
        (0x07, 2),
        (0x08, 8),
        (0x09, 2),
        (0x0a, 5),
        (0x0b, 2),
        (0x0e, 4),
        (0x10, 1),
        (0x11, 2),
        (0x12, 5),
        (0x13, 17),
        (0x14, 1),
        (0x15, 2),
        (0x17, 2),
        (0x19, 13),
        (0x1c, 5),
        (0x1d, 8),
        (0x1f, 1),
        (0x24, 1),
        (0x6a, 4),
        (0x6b, 2),
        (0xaf, 3),
        (0xb1, 2),
        (0xbc, 2),
        (0xcf, 2),
        (0xd1, 2),
        (0xd4, 12),
        (0xd5, 9),
        (0xd6, 2),
        (0xd7, 2),
        (0xda, 1),
        (0xe0, 5),
        (0xe1, 2),
        (0xe7, 4),
        (0xe8, 2),
        (0xee, 32),
        (0xf0, 4),
        (0xf8, 2),
        (0xfa, 3),
        (0xfb, 1),
    ];
    #[rustfmt::skip]
    const SINGLETONS1L: &[u8] = &[
        0x0c, 0x27, 0x3b, 0x3e, 0x4e, 0x4f, 0x8f, 0x9e,
        0x9e, 0x9f, 0x7b, 0x8b, 0x93, 0x96, 0xa2, 0xb2,
        0xba, 0x86, 0xb1, 0x06, 0x07, 0x09, 0x36, 0x3d,
        0x3e, 0x56, 0xf3, 0xd0, 0xd1, 0x04, 0x14, 0x18,
        0x36, 0x37, 0x56, 0x57, 0x7f, 0xaa, 0xae, 0xaf,
        0xbd, 0x35, 0xe0, 0x12, 0x87, 0x89, 0x8e, 0x9e,
        0x04, 0x0d, 0x0e, 0x11, 0x12, 0x29, 0x31, 0x34,
        0x3a, 0x45, 0x46, 0x49, 0x4a, 0x4e, 0x4f, 0x64,
        0x65, 0x5c, 0xb6, 0xb7, 0x1b, 0x1c, 0x07, 0x08,
        0x0a, 0x0b, 0x14, 0x17, 0x36, 0x39, 0x3a, 0xa8,
        0xa9, 0xd8, 0xd9, 0x09, 0x37, 0x90, 0x91, 0xa8,
        0x07, 0x0a, 0x3b, 0x3e, 0x66, 0x69, 0x8f, 0x92,
        0x11, 0x6f, 0x5f, 0xbf, 0xee, 0xef, 0x5a, 0x62,
        0xf4, 0xfc, 0xff, 0x53, 0x54, 0x9a, 0x9b, 0x2e,
        0x2f, 0x27, 0x28, 0x55, 0x9d, 0xa0, 0xa1, 0xa3,
        0xa4, 0xa7, 0xa8, 0xad, 0xba, 0xbc, 0xc4, 0x06,
        0x0b, 0x0c, 0x15, 0x1d, 0x3a, 0x3f, 0x45, 0x51,
        0xa6, 0xa7, 0xcc, 0xcd, 0xa0, 0x07, 0x19, 0x1a,
        0x22, 0x25, 0x3e, 0x3f, 0xe7, 0xec, 0xef, 0xff,
        0xc5, 0xc6, 0x04, 0x20, 0x23, 0x25, 0x26, 0x28,
        0x33, 0x38, 0x3a, 0x48, 0x4a, 0x4c, 0x50, 0x53,
        0x55, 0x56, 0x58, 0x5a, 0x5c, 0x5e, 0x60, 0x63,
        0x65, 0x66, 0x6b, 0x73, 0x78, 0x7d, 0x7f, 0x8a,
        0xa4, 0xaa, 0xaf, 0xb0, 0xc0, 0xd0, 0xae, 0xaf,
        0x6e, 0x6f, 0xbe, 0x93,
    ];
    #[rustfmt::skip]
    const NORMAL0: &[u8] = &[
        0x00, 0x20,
        0x5f, 0x22,
        0x82, 0xdf, 0x04,
        0x82, 0x44, 0x08,
        0x1b, 0x04,
        0x06, 0x11,
        0x81, 0xac, 0x0e,
        0x80, 0xab, 0x05,
        0x1f, 0x09,
        0x81, 0x1b, 0x03,
        0x19, 0x08,
        0x01, 0x04,
        0x2f, 0x04,
        0x34, 0x04,
        0x07, 0x03,
        0x01, 0x07,
        0x06, 0x07,
        0x11, 0x0a,
        0x50, 0x0f,
        0x12, 0x07,
        0x55, 0x07,
        0x03, 0x04,
        0x1c, 0x0a,
        0x09, 0x03,
        0x08, 0x03,
        0x07, 0x03,
        0x02, 0x03,
        0x03, 0x03,
        0x0c, 0x04,
        0x05, 0x03,
        0x0b, 0x06,
        0x01, 0x0e,
        0x15, 0x05,
        0x4e, 0x07,
        0x1b, 0x07,
        0x57, 0x07,
        0x02, 0x06,
        0x17, 0x0c,
        0x50, 0x04,
        0x43, 0x03,
        0x2d, 0x03,
        0x01, 0x04,
        0x11, 0x06,
        0x0f, 0x0c,
        0x3a, 0x04,
        0x1d, 0x25,
        0x5f, 0x20,
        0x6d, 0x04,
        0x6a, 0x25,
        0x80, 0xc8, 0x05,
        0x82, 0xb0, 0x03,
        0x1a, 0x06,
        0x82, 0xfd, 0x03,
        0x59, 0x07,
        0x16, 0x09,
        0x18, 0x09,
        0x14, 0x0c,
        0x14, 0x0c,
        0x6a, 0x06,
        0x0a, 0x06,
        0x1a, 0x06,
        0x59, 0x07,
        0x2b, 0x05,
        0x46, 0x0a,
        0x2c, 0x04,
        0x0c, 0x04,
        0x01, 0x03,
        0x31, 0x0b,
        0x2c, 0x04,
        0x1a, 0x06,
        0x0b, 0x03,
        0x80, 0xac, 0x06,
        0x0a, 0x06,
        0x2f, 0x31,
        0x4d, 0x03,
        0x80, 0xa4, 0x08,
        0x3c, 0x03,
        0x0f, 0x03,
        0x3c, 0x07,
        0x38, 0x08,
        0x2b, 0x05,
        0x82, 0xff, 0x11,
        0x18, 0x08,
        0x2f, 0x11,
        0x2d, 0x03,
        0x21, 0x0f,
        0x21, 0x0f,
        0x80, 0x8c, 0x04,
        0x82, 0x97, 0x19,
        0x0b, 0x15,
        0x88, 0x94, 0x05,
        0x2f, 0x05,
        0x3b, 0x07,
        0x02, 0x0e,
        0x18, 0x09,
        0x80, 0xbe, 0x22,
        0x74, 0x0c,
        0x80, 0xd6, 0x1a,
        0x81, 0x10, 0x05,
        0x80, 0xdf, 0x0b,
        0xf2, 0x9e, 0x03,
        0x37, 0x09,
        0x81, 0x5c, 0x14,
        0x80, 0xb8, 0x08,
        0x80, 0xcb, 0x05,
        0x0a, 0x18,
        0x3b, 0x03,
        0x0a, 0x06,
        0x38, 0x08,
        0x46, 0x08,
        0x0c, 0x06,
        0x74, 0x0b,
        0x1e, 0x03,
        0x5a, 0x04,
        0x59, 0x09,
        0x80, 0x83, 0x18,
        0x1c, 0x0a,
        0x16, 0x09,
        0x4c, 0x04,
        0x80, 0x8a, 0x06,
        0xab, 0xa4, 0x0c,
        0x17, 0x04,
        0x31, 0xa1, 0x04,
        0x81, 0xda, 0x26,
        0x07, 0x0c,
        0x05, 0x05,
        0x80, 0xa6, 0x10,
        0x81, 0xf5, 0x07,
        0x01, 0x20,
        0x2a, 0x06,
        0x4c, 0x04,
        0x80, 0x8d, 0x04,
        0x80, 0xbe, 0x03,
        0x1b, 0x03,
        0x0f, 0x0d,
    ];
    #[rustfmt::skip]
    const NORMAL1: &[u8] = &[
        0x5e, 0x22,
        0x7b, 0x05,
        0x03, 0x04,
        0x2d, 0x03,
        0x66, 0x03,
        0x01, 0x2f,
        0x2e, 0x80, 0x82,
        0x1d, 0x03,
        0x31, 0x0f,
        0x1c, 0x04,
        0x24, 0x09,
        0x1e, 0x05,
        0x2b, 0x05,
        0x44, 0x04,
        0x0e, 0x2a,
        0x80, 0xaa, 0x06,
        0x24, 0x04,
        0x24, 0x04,
        0x28, 0x08,
        0x34, 0x0b,
        0x4e, 0x43,
        0x81, 0x37, 0x09,
        0x16, 0x0a,
        0x08, 0x18,
        0x3b, 0x45,
        0x39, 0x03,
        0x63, 0x08,
        0x09, 0x30,
        0x16, 0x05,
        0x21, 0x03,
        0x1b, 0x05,
        0x01, 0x40,
        0x38, 0x04,
        0x4b, 0x05,
        0x2f, 0x04,
        0x0a, 0x07,
        0x09, 0x07,
        0x40, 0x20,
        0x27, 0x04,
        0x0c, 0x09,
        0x36, 0x03,
        0x3a, 0x05,
        0x1a, 0x07,
        0x04, 0x0c,
        0x07, 0x50,
        0x49, 0x37,
        0x33, 0x0d,
        0x33, 0x07,
        0x2e, 0x08,
        0x0a, 0x81, 0x26,
        0x52, 0x4b,
        0x2b, 0x08,
        0x2a, 0x16,
        0x1a, 0x26,
        0x1c, 0x14,
        0x17, 0x09,
        0x4e, 0x04,
        0x24, 0x09,
        0x44, 0x0d,
        0x19, 0x07,
        0x0a, 0x06,
        0x48, 0x08,
        0x27, 0x09,
        0x75, 0x0b,
        0x42, 0x3e,
        0x2a, 0x06,
        0x3b, 0x05,
        0x0a, 0x06,
        0x51, 0x06,
        0x01, 0x05,
        0x10, 0x03,
        0x05, 0x80, 0x8b,
        0x62, 0x1e,
        0x48, 0x08,
        0x0a, 0x80, 0xa6,
        0x5e, 0x22,
        0x45, 0x0b,
        0x0a, 0x06,
        0x0d, 0x13,
        0x3a, 0x06,
        0x0a, 0x36,
        0x2c, 0x04,
        0x17, 0x80, 0xb9,
        0x3c, 0x64,
        0x53, 0x0c,
        0x48, 0x09,
        0x0a, 0x46,
        0x45, 0x1b,
        0x48, 0x08,
        0x53, 0x0d,
        0x49, 0x07,
        0x0a, 0x80, 0xf6,
        0x46, 0x0a,
        0x1d, 0x03,
        0x47, 0x49,
        0x37, 0x03,
        0x0e, 0x08,
        0x0a, 0x06,
        0x39, 0x07,
        0x0a, 0x81, 0x36,
        0x19, 0x07,
        0x3b, 0x03,
        0x1c, 0x56,
        0x01, 0x0f,
        0x32, 0x0d,
        0x83, 0x9b, 0x66,
        0x75, 0x0b,
        0x80, 0xc4, 0x8a, 0x4c,
        0x63, 0x0d,
        0x84, 0x30, 0x10,
        0x16, 0x8f, 0xaa,
        0x82, 0x47, 0xa1, 0xb9,
        0x82, 0x39, 0x07,
        0x2a, 0x04,
        0x5c, 0x06,
        0x26, 0x0a,
        0x46, 0x0a,
        0x28, 0x05,
        0x13, 0x82, 0xb0,
        0x5b, 0x65,
        0x4b, 0x04,
        0x39, 0x07,
        0x11, 0x40,
        0x05, 0x0b,
        0x02, 0x0e,
        0x97, 0xf8, 0x08,
        0x84, 0xd6, 0x2a,
        0x09, 0xa2, 0xe7,
        0x81, 0x33, 0x0f,
        0x01, 0x1d,
        0x06, 0x0e,
        0x04, 0x08,
        0x81, 0x8c, 0x89, 0x04,
        0x6b, 0x05,
        0x0d, 0x03,
        0x09, 0x07,
        0x10, 0x92, 0x60,
        0x47, 0x09,
        0x74, 0x3c,
        0x80, 0xf6, 0x0a,
        0x73, 0x08,
        0x70, 0x15,
        0x46, 0x7a,
        0x14, 0x0c,
        0x14, 0x0c,
        0x57, 0x09,
        0x19, 0x80, 0x87,
        0x81, 0x47, 0x03,
        0x85, 0x42, 0x0f,
        0x15, 0x84, 0x50,
        0x1f, 0x06,
        0x06, 0x80, 0xd5,
        0x2b, 0x05,
        0x3e, 0x21,
        0x01, 0x70,
        0x2d, 0x03,
        0x1a, 0x04,
        0x02, 0x81, 0x40,
        0x1f, 0x11,
        0x3a, 0x05,
        0x01, 0x81, 0xd0,
        0x2a, 0x82, 0xe6,
        0x80, 0xf7, 0x29,
        0x4c, 0x04,
        0x0a, 0x04,
        0x02, 0x83, 0x11,
        0x44, 0x4c,
        0x3d, 0x80, 0xc2,
        0x3c, 0x06,
        0x01, 0x04,
        0x55, 0x05,
        0x1b, 0x34,
        0x02, 0x81, 0x0e,
        0x2c, 0x04,
        0x64, 0x0c,
        0x56, 0x0a,
        0x80, 0xae, 0x38,
        0x1d, 0x0d,
        0x2c, 0x04,
        0x09, 0x07,
        0x02, 0x0e,
        0x06, 0x80, 0x9a,
        0x83, 0xd8, 0x04,
        0x11, 0x03,
        0x0d, 0x03,
        0x77, 0x04,
        0x5f, 0x06,
        0x0c, 0x04,
        0x01, 0x0f,
        0x0c, 0x04,
        0x38, 0x08,
        0x0a, 0x06,
        0x28, 0x08,
        0x22, 0x4e,
        0x81, 0x54, 0x0c,
        0x1d, 0x03,
        0x09, 0x07,
        0x36, 0x08,
        0x0e, 0x04,
        0x09, 0x07,
        0x09, 0x07,
        0x80, 0xcb, 0x25,
        0x0a, 0x84, 0x06,
    ];
}

// SOURCE: https://github.com/rust-lang/rust/blob/1.81.0/library/core/src/unicode/unicode_data.rs
// INITIAL-MODIFICATION: Rename mod
#[rustfmt::skip]
mod rust_core_1_81_grapheme_extend {
    static SHORT_OFFSET_RUNS: [u32; 33] = [
        768, 2098307, 6292881, 10490717, 522196754, 526393356, 731917551, 740306986, 752920175,
        761309186, 778107678, 908131840, 912326558, 920715773, 924912129, 937495844, 962662059,
        966858799, 1214323760, 1285627635, 1348547648, 1369533168, 1377922895, 1386331293,
        1398918912, 1403113829, 1411504640, 1440866304, 1466032814, 1495393516, 1503783120,
        1508769824, 1518273008,
    ];
    static OFFSETS: [u8; 727] = [
        0, 112, 0, 7, 0, 45, 1, 1, 1, 2, 1, 2, 1, 1, 72, 11, 48, 21, 16, 1, 101, 7, 2, 6, 2, 2, 1,
        4, 35, 1, 30, 27, 91, 11, 58, 9, 9, 1, 24, 4, 1, 9, 1, 3, 1, 5, 43, 3, 60, 8, 42, 24, 1, 32,
        55, 1, 1, 1, 4, 8, 4, 1, 3, 7, 10, 2, 29, 1, 58, 1, 1, 1, 2, 4, 8, 1, 9, 1, 10, 2, 26, 1, 2,
        2, 57, 1, 4, 2, 4, 2, 2, 3, 3, 1, 30, 2, 3, 1, 11, 2, 57, 1, 4, 5, 1, 2, 4, 1, 20, 2, 22, 6,
        1, 1, 58, 1, 1, 2, 1, 4, 8, 1, 7, 3, 10, 2, 30, 1, 59, 1, 1, 1, 12, 1, 9, 1, 40, 1, 3, 1,
        55, 1, 1, 3, 5, 3, 1, 4, 7, 2, 11, 2, 29, 1, 58, 1, 2, 1, 2, 1, 3, 1, 5, 2, 7, 2, 11, 2, 28,
        2, 57, 2, 1, 1, 2, 4, 8, 1, 9, 1, 10, 2, 29, 1, 72, 1, 4, 1, 2, 3, 1, 1, 8, 1, 81, 1, 2, 7,
        12, 8, 98, 1, 2, 9, 11, 7, 73, 2, 27, 1, 1, 1, 1, 1, 55, 14, 1, 5, 1, 2, 5, 11, 1, 36, 9, 1,
        102, 4, 1, 6, 1, 2, 2, 2, 25, 2, 4, 3, 16, 4, 13, 1, 2, 2, 6, 1, 15, 1, 0, 3, 0, 3, 29, 2,
        30, 2, 30, 2, 64, 2, 1, 7, 8, 1, 2, 11, 9, 1, 45, 3, 1, 1, 117, 2, 34, 1, 118, 3, 4, 2, 9,
        1, 6, 3, 219, 2, 2, 1, 58, 1, 1, 7, 1, 1, 1, 1, 2, 8, 6, 10, 2, 1, 48, 31, 49, 4, 48, 7, 1,
        1, 5, 1, 40, 9, 12, 2, 32, 4, 2, 2, 1, 3, 56, 1, 1, 2, 3, 1, 1, 3, 58, 8, 2, 2, 152, 3, 1,
        13, 1, 7, 4, 1, 6, 1, 3, 2, 198, 64, 0, 1, 195, 33, 0, 3, 141, 1, 96, 32, 0, 6, 105, 2, 0,
        4, 1, 10, 32, 2, 80, 2, 0, 1, 3, 1, 4, 1, 25, 2, 5, 1, 151, 2, 26, 18, 13, 1, 38, 8, 25, 11,
        46, 3, 48, 1, 2, 4, 2, 2, 39, 1, 67, 6, 2, 2, 2, 2, 12, 1, 8, 1, 47, 1, 51, 1, 1, 3, 2, 2,
        5, 2, 1, 1, 42, 2, 8, 1, 238, 1, 2, 1, 4, 1, 0, 1, 0, 16, 16, 16, 0, 2, 0, 1, 226, 1, 149,
        5, 0, 3, 1, 2, 5, 4, 40, 3, 4, 1, 165, 2, 0, 4, 0, 2, 80, 3, 70, 11, 49, 4, 123, 1, 54, 15,
        41, 1, 2, 2, 10, 3, 49, 4, 2, 2, 7, 1, 61, 3, 36, 5, 1, 8, 62, 1, 12, 2, 52, 9, 10, 4, 2, 1,
        95, 3, 2, 1, 1, 2, 6, 1, 2, 1, 157, 1, 3, 8, 21, 2, 57, 2, 1, 1, 1, 1, 22, 1, 14, 7, 3, 5,
        195, 8, 2, 3, 1, 1, 23, 1, 81, 1, 2, 6, 1, 1, 2, 1, 1, 2, 1, 2, 235, 1, 2, 4, 6, 2, 1, 2,
        27, 2, 85, 8, 2, 1, 1, 2, 106, 1, 1, 1, 2, 6, 1, 1, 101, 3, 2, 4, 1, 5, 0, 9, 1, 2, 245, 1,
        10, 2, 1, 1, 4, 1, 144, 4, 2, 2, 4, 1, 32, 10, 40, 6, 2, 4, 8, 1, 9, 6, 2, 3, 46, 13, 1, 2,
        0, 7, 1, 6, 1, 1, 82, 22, 2, 7, 1, 2, 1, 2, 122, 6, 3, 1, 1, 2, 1, 7, 1, 1, 72, 2, 3, 1, 1,
        1, 0, 2, 11, 2, 52, 5, 5, 1, 1, 1, 0, 1, 6, 15, 0, 5, 59, 7, 0, 1, 63, 4, 81, 1, 0, 2, 0,
        46, 2, 23, 0, 1, 1, 3, 4, 5, 8, 8, 2, 7, 30, 4, 148, 3, 0, 55, 4, 50, 8, 1, 14, 1, 22, 5, 1,
        15, 0, 7, 1, 17, 2, 7, 1, 2, 1, 5, 100, 1, 160, 7, 0, 1, 61, 4, 0, 4, 0, 7, 109, 7, 0, 96,
        128, 240, 0,
    ];

    #[inline]
    pub fn lookup(c: char) -> bool {
        (c as u32) >= 0x300 && lookup_slow(c)
    }

    fn lookup_slow(c: char) -> bool {
        skip_search(
            c as u32,
            &SHORT_OFFSET_RUNS,
            &OFFSETS,
        )
    }

    #[inline(always)]
    fn skip_search<const SOR: usize, const OFFSETS: usize>(
        needle: u32,
        short_offset_runs: &[u32; SOR],
        offsets: &[u8; OFFSETS],
    ) -> bool {
        // Note that this *cannot* be past the end of the array, as the last
        // element is greater than std::char::MAX (the largest possible needle).
        //
        // So, we cannot have found it (i.e. Ok(idx) + 1 != length) and the correct
        // location cannot be past it, so Err(idx) != length either.
        //
        // This means that we can avoid bounds checking for the accesses below, too.
        let last_idx =
            match short_offset_runs.binary_search_by_key(&(needle << 11), |header| header << 11) {
                Ok(idx) => idx + 1,
                Err(idx) => idx,
            };

        let mut offset_idx = decode_length(short_offset_runs[last_idx]);
        let length = if let Some(next) = short_offset_runs.get(last_idx + 1) {
            decode_length(*next) - offset_idx
        } else {
            offsets.len() - offset_idx
        };
        let prev =
            last_idx.checked_sub(1).map(|prev| decode_prefix_sum(short_offset_runs[prev])).unwrap_or(0);

        let total = needle - prev;
        let mut prefix_sum = 0;
        for _ in 0..(length - 1) {
            let offset = offsets[offset_idx];
            prefix_sum += offset as u32;
            if prefix_sum > total {
                break;
            }
            offset_idx += 1;
        }
        offset_idx % 2 == 1
    }

    fn decode_length(short_offset_run_header: u32) -> usize {
        (short_offset_run_header >> 21) as usize
    }

    fn decode_prefix_sum(short_offset_run_header: u32) -> u32 {
        short_offset_run_header & ((1 << 21) - 1)
    }
}
