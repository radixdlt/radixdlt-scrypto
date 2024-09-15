use crate::internal_prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ManifestIdAllocator {
    next_bucket_id: u32,
    next_proof_id: u32,
    next_address_reservation_id: u32,
    next_address_id: u32,
    next_intent_id: u32,
}

impl ManifestIdAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    // NOTE: Overflow of untrusted inputs is impossible:
    // * MAX_TRANSACTION_SIZE is 1MB (and this check happens first)
    // * Each instruction takes more than 1 byte
    // * u32 accepts more than 1M ~ 2^20

    pub fn new_bucket_id(&mut self) -> ManifestBucket {
        let id = self.next_bucket_id;
        self.next_bucket_id += 1;
        ManifestBucket(id)
    }

    pub fn new_proof_id(&mut self) -> ManifestProof {
        let id = self.next_proof_id;
        self.next_proof_id += 1;
        ManifestProof(id)
    }

    pub fn new_address_reservation_id(&mut self) -> ManifestAddressReservation {
        let id = self.next_address_reservation_id;
        self.next_address_reservation_id += 1;
        ManifestAddressReservation(id)
    }

    pub fn new_address_id(&mut self) -> ManifestNamedAddress {
        let id = self.next_address_id;
        self.next_address_id += 1;
        ManifestNamedAddress(id)
    }

    pub fn new_named_intent_id(&mut self) -> ManifestNamedIntent {
        let id = self.next_intent_id;
        self.next_intent_id += 1;
        ManifestNamedIntent(id)
    }
}
