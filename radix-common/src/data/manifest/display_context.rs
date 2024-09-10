use super::model::*;
use crate::address::AddressBech32Encoder;
use sbor::rust::prelude::*;

/// Note - this is quite similar to ManifestDecompilationDisplayContext
/// - except this is used with formatting of an encoded payload, rather than a ManifestValue itself
#[derive(Clone, Copy, Debug, Default)]
pub struct ManifestValueDisplayContext<'a> {
    pub address_bech32_encoder: Option<&'a AddressBech32Encoder>,
    pub bucket_names: Option<&'a NonIterMap<ManifestBucket, String>>,
    pub proof_names: Option<&'a NonIterMap<ManifestProof, String>>,
    pub address_reservation_names: Option<&'a NonIterMap<ManifestAddressReservation, String>>,
    pub address_names: Option<&'a NonIterMap<ManifestNamedAddress, String>>,
}

impl<'a> ManifestValueDisplayContext<'a> {
    pub fn no_context() -> Self {
        Self::default()
    }

    pub fn with_optional_bech32(address_bech32_encoder: Option<&'a AddressBech32Encoder>) -> Self {
        Self {
            address_bech32_encoder,
            ..Default::default()
        }
    }

    pub fn with_bech32_and_names(
        address_bech32_encoder: Option<&'a AddressBech32Encoder>,
        bucket_names: &'a NonIterMap<ManifestBucket, String>,
        proof_names: &'a NonIterMap<ManifestProof, String>,
        address_reservation_names: &'a NonIterMap<ManifestAddressReservation, String>,
        address_names: &'a NonIterMap<ManifestNamedAddress, String>,
    ) -> Self {
        Self {
            address_bech32_encoder,
            bucket_names: Some(bucket_names),
            proof_names: Some(proof_names),
            address_reservation_names: Some(address_reservation_names),
            address_names: Some(address_names),
        }
    }

    pub fn get_bucket_name(&self, bucket_id: &ManifestBucket) -> Option<&str> {
        self.bucket_names
            .and_then(|names| names.get(bucket_id).map(|s| s.as_str()))
    }

    pub fn get_proof_name(&self, proof_id: &ManifestProof) -> Option<&str> {
        self.proof_names
            .and_then(|names| names.get(proof_id).map(|s| s.as_str()))
    }

    pub fn get_address_reservation_name(
        &self,
        address_reservation_id: &ManifestAddressReservation,
    ) -> Option<&str> {
        self.address_reservation_names
            .and_then(|names| names.get(address_reservation_id).map(|s| s.as_str()))
    }

    pub fn get_address_name(&self, address_id: &ManifestNamedAddress) -> Option<&str> {
        self.address_names
            .and_then(|names| names.get(address_id).map(|s| s.as_str()))
    }
}

impl<'a> Into<ManifestValueDisplayContext<'a>> for &'a AddressBech32Encoder {
    fn into(self) -> ManifestValueDisplayContext<'a> {
        ManifestValueDisplayContext::with_optional_bech32(Some(self))
    }
}

impl<'a> Into<ManifestValueDisplayContext<'a>> for Option<&'a AddressBech32Encoder> {
    fn into(self) -> ManifestValueDisplayContext<'a> {
        ManifestValueDisplayContext::with_optional_bech32(self)
    }
}
