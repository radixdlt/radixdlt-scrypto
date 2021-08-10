use crate::dependencies::*;

/// Represents an address.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub enum DecodeAddressError {
    InvalidHex(hex::FromHexError),
    InvalidLength,
    InvalidType(u8),
}

fn from_slice<const N: usize>(slice: &[u8]) -> Result<[u8; N], DecodeAddressError> {
    if slice.len() == N {
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(&slice[0..N]);
        Ok(bytes)
    } else {
        Err(DecodeAddressError::InvalidLength)
    }
}

impl Address {
    /// Decode an address from its hex representation.
    pub fn from_hex(hex: &str) -> Result<Self, DecodeAddressError> {
        let bytes = hex::decode(hex).map_err(|e| DecodeAddressError::InvalidHex(e))?;

        Self::from_slice(&bytes)
    }

    /// Decode an address from a slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DecodeAddressError> {
        let invalid_len = DecodeAddressError::InvalidLength;
        if bytes.len() >= 1 {
            let kind = bytes[0];
            let data = &bytes[1..bytes.len()];
            match kind {
                0x00 => Ok(Address::System),
                0x01 => Ok(Address::RadixToken),
                0x03 => Ok(Address::Resource(from_slice(data)?)),
                0x04 => Ok(Address::Account(from_slice(data)?)),
                0x05 => Ok(Address::Blueprint(from_slice(data)?)),
                0x06 => Ok(Address::Component(from_slice(data)?)),
                _ => Err(DecodeAddressError::InvalidType(kind)),
            }
        } else {
            Err(invalid_len)
        }
    }
}

impl<T: AsRef<str>> From<T> for Address {
    fn from(s: T) -> Self {
        Self::from_hex(s.as_ref()).unwrap()
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

impl Into<Vec<u8>> for Address {
    fn into(self) -> Vec<u8> {
        let mut buf = vec![];
        match self {
            Self::System => push!(buf, 0x00),
            Self::RadixToken => push!(buf, 0x01),
            Self::Resource(d) => push!(buf, 0x03, d),
            Self::Account(d) => push!(buf, 0x04, d),
            Self::Blueprint(d) => push!(buf, 0x05, d),
            Self::Component(d) => push!(buf, 0x06, d),
        }
        buf
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        let bytes: Vec<u8> = self.clone().into();
        hex::encode(bytes)
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::dependencies::*;
    use crate::*;

    #[test]
    fn test_from_to_string() {
        let s = "040377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee9443eafc92106bba";
        let a: Address = s.into();
        assert_eq!(a.to_string(), s);
    }
}
