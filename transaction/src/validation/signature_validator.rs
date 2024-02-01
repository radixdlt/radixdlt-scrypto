use crate::internal_prelude::*;

pub fn recover(signed_hash: &Hash, signature: &SignatureWithPublicKeyV1) -> Option<PublicKey> {
    match signature {
        SignatureWithPublicKeyV1::Secp256k1 { signature } => {
            recover_secp256k1(signed_hash, signature).map(Into::into)
        }
        SignatureWithPublicKeyV1::Ed25519 { public_key, .. } => Some(public_key.clone().into()),
    }
}

pub fn verify(signed_hash: &Hash, public_key: &PublicKey, signature: &SignatureV1) -> bool {
    match (public_key, signature) {
        (PublicKey::Secp256k1(pk), SignatureV1::Secp256k1(sig)) => {
            verify_secp256k1(&signed_hash, pk, sig)
        }
        (PublicKey::Ed25519(pk), SignatureV1::Ed25519(sig)) => {
            verify_ed25519(&signed_hash, pk, sig)
        }
        _ => false,
    }
}
