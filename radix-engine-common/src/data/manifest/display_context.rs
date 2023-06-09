use super::model::*;
use crate::address::Bech32Encoder;
use sbor::rust::prelude::*;

/// Note - this is quite similar to ManifestDecompilationDisplayContext
/// - except this is used with formatting of an encoded payload, rather than a ManifestValue itself
#[derive(Clone, Copy, Debug, Default)]
pub struct ManifestValueDisplayContext<'a> {
    pub bech32_encoder: Option<&'a Bech32Encoder>,
    pub bucket_names: Option<&'a NonIterMap<ManifestBucket, String>>,
    pub proof_names: Option<&'a NonIterMap<ManifestProof, String>>,
    pub reservation_names: Option<&'a NonIterMap<ManifestReservation, String>>,
    pub named_address_names: Option<&'a NonIterMap<ManifestNamedAddress, String>>,
}

impl<'a> ManifestValueDisplayContext<'a> {
    pub fn no_context() -> Self {
        Self::default()
    }

    pub fn with_optional_bech32(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            ..Default::default()
        }
    }

    pub fn with_bech32_and_names(
        bech32_encoder: Option<&'a Bech32Encoder>,
        bucket_names: &'a NonIterMap<ManifestBucket, String>,
        proof_names: &'a NonIterMap<ManifestProof, String>,
        reservation_names: &'a NonIterMap<ManifestReservation, String>,
        named_address_names: &'a NonIterMap<ManifestNamedAddress, String>,
    ) -> Self {
        Self {
            bech32_encoder,
            bucket_names: Some(bucket_names),
            proof_names: Some(proof_names),
            reservation_names: Some(reservation_names),
            named_address_names: Some(named_address_names),
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

    pub fn get_reservation_name(&self, reservation_id: &ManifestReservation) -> Option<&str> {
        self.reservation_names
            .and_then(|names| names.get(reservation_id).map(|s| s.as_str()))
    }

    pub fn get_named_address_name(&self, named_address_id: &ManifestNamedAddress) -> Option<&str> {
        self.named_address_names
            .and_then(|names| names.get(named_address_id).map(|s| s.as_str()))
    }
}

impl<'a> Into<ManifestValueDisplayContext<'a>> for &'a Bech32Encoder {
    fn into(self) -> ManifestValueDisplayContext<'a> {
        ManifestValueDisplayContext::with_optional_bech32(Some(self))
    }
}

impl<'a> Into<ManifestValueDisplayContext<'a>> for Option<&'a Bech32Encoder> {
    fn into(self) -> ManifestValueDisplayContext<'a> {
        ManifestValueDisplayContext::with_optional_bech32(self)
    }
}
