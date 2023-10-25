use scrypto::prelude::*;

#[blueprint]
mod bls {
    struct BLS {}

    impl BLS {
        pub fn verify(messages: Vec<Vec<u8>>, public_keys: Vec<Vec<u8>>, sig: Vec<u8>) -> bool {
            use bls_signatures::*;

            let hashes = messages
                .iter()
                .map(|message| hash(message))
                .collect::<Vec<_>>();
            let public_keys: Vec<PublicKey> = public_keys
                .into_iter()
                .map(|x| PublicKey::from_bytes(&x).unwrap())
                .collect();
            let sig = Signature::from_bytes(&sig).unwrap();
            verify(&sig, &hashes, &public_keys)
        }
    }
}
