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

impl<'a> Into<AddressDisplayContext<'a>> for &'a Bech32Encoder {
    fn into(self) -> AddressDisplayContext<'a> {
        AddressDisplayContext::with_encoder(self)
    }
}

impl<'a> Into<AddressDisplayContext<'a>> for Option<&'a Bech32Encoder> {
    fn into(self) -> AddressDisplayContext<'a> {
        AddressDisplayContext { encoder: self }
    }
}
