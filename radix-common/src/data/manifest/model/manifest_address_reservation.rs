use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct ManifestAddressReservation(pub u32);

//========
// resolution
//========

/// This is for use with the [`ResolvableManifestAddressReservation`] trait.
/// Implementers should panic if a proof cannot be found.
pub trait NamedManifestAddressReservationResolver {
    fn assert_address_reservation_exists(&self, address_reservation: ManifestAddressReservation);
    fn resolve_address_reservation(&self, name: &str) -> ManifestAddressReservation;
}

/// This trait is intended to be used as an `impl` argument in helper methods
/// operating on manifests, to resolve a [`ManifestAddressReservation`] from a name, an id,
/// or from itself.
///
/// The resolution process relies on a [`NamedManifestAddressReservationResolver`] which can
/// provide a lookup to/from names.
pub trait ResolvableManifestAddressReservation {
    fn resolve(
        self,
        resolver: &impl NamedManifestAddressReservationResolver,
    ) -> ManifestAddressReservation;
}

impl<A, E> ResolvableManifestAddressReservation for A
where
    A: TryInto<ManifestAddressReservation, Error = E>,
    E: Debug,
{
    fn resolve(
        self,
        resolver: &impl NamedManifestAddressReservationResolver,
    ) -> ManifestAddressReservation {
        let address_reservation = self
            .try_into()
            .expect("Value was not a valid ManifestProof");
        resolver.assert_address_reservation_exists(address_reservation);
        address_reservation
    }
}

impl<'a> ResolvableManifestAddressReservation for &'a str {
    fn resolve(
        self,
        resolver: &impl NamedManifestAddressReservationResolver,
    ) -> ManifestAddressReservation {
        resolver.resolve_address_reservation(self).into()
    }
}

impl<'a> ResolvableManifestAddressReservation for &'a String {
    fn resolve(
        self,
        resolver: &impl NamedManifestAddressReservationResolver,
    ) -> ManifestAddressReservation {
        resolver.resolve_address_reservation(self.as_str()).into()
    }
}

impl ResolvableManifestAddressReservation for String {
    fn resolve(
        self,
        resolver: &impl NamedManifestAddressReservationResolver,
    ) -> ManifestAddressReservation {
        resolver.resolve_address_reservation(self.as_str()).into()
    }
}

/// This trait is intended to be used as an `impl` argument in helper methods
/// operating on manifests, to resolve an [`Option<ManifestAddressReservation>`] from a name, an id,
/// a [`ManifestAddressReservation`], or `None`.
///
/// The resolution process relies on a [`NamedManifestAddressReservationResolver`] which can
/// provide a lookup to/from names.
pub trait ResolvableOptionalManifestAddressReservation {
    fn resolve(
        self,
        resolver: &impl NamedManifestAddressReservationResolver,
    ) -> Option<ManifestAddressReservation>;
}

impl<T: ResolvableManifestAddressReservation> ResolvableOptionalManifestAddressReservation for T {
    fn resolve(
        self,
        resolver: &impl NamedManifestAddressReservationResolver,
    ) -> Option<ManifestAddressReservation> {
        Some(<Self as ResolvableManifestAddressReservation>::resolve(
            self, resolver,
        ))
    }
}

// We only implement it for one Option, so that `None` has a unique implementation
// We choose Option<String> for backwards compatibility
impl ResolvableOptionalManifestAddressReservation for Option<String> {
    fn resolve(
        self,
        resolver: &impl NamedManifestAddressReservationResolver,
    ) -> Option<ManifestAddressReservation> {
        self.map(|r| <String as ResolvableManifestAddressReservation>::resolve(r, resolver))
    }
}

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
