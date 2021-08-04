extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use core::convert::TryInto;
use core::fmt;

use sbor::{Decode, Encode};

use crate::utils::*;

/// Represents an address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum Address {
    /// System address.
    System,

    /// Radix native token address.
    RadixToken,

    /// Resource address.
    Resource([u8; 26]),

    /// Public key account.
    Account([u8; 33]),

    /// Published Scrypto blueprint.
    Blueprint([u8; 26]),

    /// Instantiated Scrypto component.
    Component([u8; 26]),
}

impl Address {
    /// Decode an address from its hex representation.
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex_decode(hex)?;

        let e = format!("Invalid address: {}", hex);
        if bytes.len() >= 1 {
            let kind = bytes[0];
            let data = &bytes[1..bytes.len()];
            match kind {
                0x00 => Ok(Address::System),
                0x01 => Ok(Address::RadixToken),
                0x03 => Ok(Address::Resource(data.try_into().map_err(|_| e)?)),
                0x04 => Ok(Address::Account(data.try_into().map_err(|_| e)?)),
                0x05 => Ok(Address::Blueprint(data.try_into().map_err(|_| e)?)),
                0x06 => Ok(Address::Component(data.try_into().map_err(|_| e)?)),
                _ => Err(e),
            }
        } else {
            Err(e)
        }
    }
}

macro_rules! push {
    ($buf: expr, $kind: expr) => {{
        $buf.push($kind);
    }};
    ($buf: expr, $kind: expr, $id: expr) => {{
        $buf.push($kind);
        $buf.extend($id);
    }};
}

impl<T: AsRef<str>> From<T> for Address {
    fn from(s: T) -> Self {
        Self::from_hex(s.as_ref()).unwrap()
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        let mut buf = vec![];
        match self {
            Self::System => push!(buf, 0x00),
            Self::RadixToken => push!(buf, 0x01),
            Self::Resource(d) => push!(buf, 0x03, d),
            Self::Account(d) => push!(buf, 0x04, d),
            Self::Blueprint(d) => push!(buf, 0x05, d),
            Self::Component(d) => push!(buf, 0x06, d),
        }
        hex_encode(buf)
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_from_to_string() {
        let s = "040377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee9443eafc92106bba";
        let a: Address = s.into();
        assert_eq!(a.to_string(), s);
    }
}
