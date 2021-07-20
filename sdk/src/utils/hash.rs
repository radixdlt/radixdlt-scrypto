use sha2::{Digest, Sha256};

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut instance = Sha256::new();
    instance.update(data);
    let result = instance.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(result.as_slice());
    hash
}
