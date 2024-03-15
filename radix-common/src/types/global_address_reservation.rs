use crate::data::scrypto::model::Own;
use crate::data::scrypto::ScryptoCustomTypeKind;
use crate::data::scrypto::ScryptoCustomValueKind;
use crate::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_common::data::scrypto::*;
use sbor::rust::prelude::*;
use sbor::*;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct GlobalAddressReservation(pub Own);

impl Categorize<ScryptoCustomValueKind> for GlobalAddressReservation {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        Own::value_kind()
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E>
    for GlobalAddressReservation
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for GlobalAddressReservation
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

impl Describe<ScryptoCustomTypeKind> for GlobalAddressReservation {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_GLOBAL_ADDRESS_RESERVATION_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        well_known_scrypto_custom_types::own_global_address_reservation_type_data()
    }
}
