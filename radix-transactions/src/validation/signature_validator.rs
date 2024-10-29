use crate::internal_prelude::*;

pub fn verify_and_recover(
    signed_hash: &Hash,
    signature: &SignatureWithPublicKeyV1,
) -> Option<PublicKey> {
    match signature {
        SignatureWithPublicKeyV1::Secp256k1 { signature } => {
            verify_and_recover_secp256k1(signed_hash, signature).map(Into::into)
        }
        SignatureWithPublicKeyV1::Ed25519 {
            public_key,
            signature,
        } => {
            if verify_ed25519(&signed_hash, public_key, signature) {
                Some(public_key.clone().into())
            } else {
                None
            }
        }
    }
}

pub fn verify(signed_hash: &Hash, public_key: &PublicKey, signature: &SignatureV1) -> bool {
    match (public_key, signature) {
        (PublicKey::Secp256k1(public_key), SignatureV1::Secp256k1(signature)) => {
            verify_secp256k1(&signed_hash, public_key, signature)
        }
        (PublicKey::Ed25519(public_key), SignatureV1::Ed25519(signature)) => {
            verify_ed25519(&signed_hash, public_key, signature)
        }
        _ => false,
    }
}
