use radix_engine_common::prelude::*;

pub trait ClientCryptoUtilsApi<E> {
    fn bls12381_v1_verify(
        &mut self,
        message: &[u8],
        public_key: &Bls12381G1PublicKey,
        signature: &Bls12381G2Signature,
    ) -> Result<u32, E>;

    fn bls12381_v1_aggregate_verify(
        &mut self,
        pub_keys_and_msgs: &[(Bls12381G1PublicKey, Vec<u8>)],
        signature: &Bls12381G2Signature,
    ) -> Result<u32, E>;

    fn bls12381_v1_fast_aggregate_verify(
        &mut self,
        message: &[u8],
        public_keys: &[Bls12381G1PublicKey],
        signature: &Bls12381G2Signature,
    ) -> Result<u32, E>;

    fn bls12381_g2_signature_aggregate(
        &mut self,
        signatures: &[Bls12381G2Signature],
    ) -> Result<Bls12381G2Signature, E>;

    fn keccak256_hash(&mut self, data: &[u8]) -> Result<Hash, E>;
}
