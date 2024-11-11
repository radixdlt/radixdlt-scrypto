use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct ManifestAddressReservation(pub u32);

labelled_resolvable_with_identity_impl!(ManifestAddressReservation, resolver_output: Self);

//========
// error
//========

/// Represents an error when parsing ManifestAddressReservation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestAddressReservationError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestAddressReservationError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestAddressReservationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestAddressReservation {
    type Error = ParseManifestAddressReservationError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 4 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(u32::from_le_bytes(slice.try_into().unwrap())))
    }
}

impl ManifestAddressReservation {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

manifest_type!(
    ManifestAddressReservation,
    ManifestCustomValueKind::AddressReservation,
    4
);
scrypto_describe_for_manifest_type!(
    ManifestAddressReservation,
    OWN_GLOBAL_ADDRESS_RESERVATION_TYPE,
    own_global_address_reservation_type_data,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_address_reservation_fail() {
        let address = ManifestAddressReservation(0);
        let mut address_vec = address.to_vec();

        assert!(ManifestAddressReservation::try_from(address_vec.as_slice()).is_ok());

        // malform encoded vector
        address_vec.push(0);
        let address_out = ManifestAddressReservation::try_from(address_vec.as_slice());
        assert_matches!(
            address_out,
            Err(ParseManifestAddressReservationError::InvalidLength)
        );

        #[cfg(not(feature = "alloc"))]
        println!(
            "Manifest Address Reservation error: {}",
            address_out.unwrap_err()
        );
    }

    #[test]
    fn manifest_address_reservation_encode_decode_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        let malformed_value: u8 = 1; // use u8 instead of u32 should inovke an error
        encoder.write_slice(&malformed_value.to_le_bytes()).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder.decode_deeper_body_with_value_kind::<ManifestAddressReservation>(
            ManifestAddressReservation::value_kind(),
        );

        // expecting 4 bytes, found only 1, so Buffer Underflow error should occur
        assert_matches!(addr_output, Err(DecodeError::BufferUnderflow { .. }));
    }
}
