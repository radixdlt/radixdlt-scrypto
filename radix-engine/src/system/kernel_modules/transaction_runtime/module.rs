use crate::kernel::module::KernelModule;
use crate::types::*;
use radix_engine_interface::crypto::Hash;

#[derive(Debug, Clone)]
pub struct TransactionRuntimeModule {
    pub tx_hash: Hash,
    pub next_id: u32,
}

impl TransactionRuntimeModule {
    pub fn transaction_hash(&self) -> Hash {
        self.tx_hash
    }

    pub fn generate_uuid(&mut self) -> u128 {
        // Take the lower 16 bytes
        let mut temp = self.tx_hash.lower_16_bytes();

        // Put TX runtime counter to the last 4 bytes.
        temp[12..16].copy_from_slice(&self.next_id.to_be_bytes());

        // Construct UUID v4 variant 1
        let uuid = (u128::from_be_bytes(temp) & 0xffffffff_ffff_0fff_3fff_ffffffffffffu128)
            | 0x00000000_0000_4000_8000_000000000000u128;

        self.next_id += 1;

        uuid
    }
}

impl KernelModule for TransactionRuntimeModule {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_gen() {
        let mut id = TransactionRuntimeModule {
            tx_hash: Hash::from_str(
                "71f26aab5eec6679f67c71211aba9a3486cc8d24194d339385ee91ee5ca7b30d",
            )
            .unwrap(),
            next_id: 5,
        };
        assert_eq!(
            NonFungibleLocalId::uuid(id.generate_uuid())
                .unwrap()
                .to_string(),
            "{86cc8d24-194d-4393-85ee-91ee00000005}"
        );

        let mut id = TransactionRuntimeModule {
            tx_hash: Hash([0u8; 32]),
            next_id: 5,
        };
        assert_eq!(
            NonFungibleLocalId::uuid(id.generate_uuid())
                .unwrap()
                .to_string(),
            "{00000000-0000-4000-8000-000000000005}"
        );

        let mut id = TransactionRuntimeModule {
            tx_hash: Hash([255u8; 32]),
            next_id: 5,
        };
        assert_eq!(
            NonFungibleLocalId::uuid(id.generate_uuid())
                .unwrap()
                .to_string(),
            "{ffffffff-ffff-4fff-bfff-ffff00000005}"
        );
    }
}
