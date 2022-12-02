use super::*;

#[derive(Clone, Copy)]
pub struct AddressDisplayContext<'a> {
    pub encoder: Option<&'a Bech32Encoder>,
}

impl<'a> AddressDisplayContext<'a> {
    pub fn with_encoder(encoder: &'a Bech32Encoder) -> Self {
        AddressDisplayContext {
            encoder: Some(encoder),
        }
    }
}

pub static NO_NETWORK: AddressDisplayContext = AddressDisplayContext { encoder: None };

impl<'a> From<&'a Bech32Encoder> for AddressDisplayContext<'a> {
    fn from(encoder: &'a Bech32Encoder) -> Self {
        Self::with_encoder(encoder)
    }
}

impl<'a> From<Option<&'a Bech32Encoder>> for AddressDisplayContext<'a> {
    fn from(encoder: Option<&'a Bech32Encoder>) -> Self {
        Self { encoder }
    }
}
