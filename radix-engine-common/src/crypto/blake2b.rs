use crate::crypto::*;
use blake2::digest::{consts::U32, Digest};
use blake2::Blake2b;

pub fn blake2b_256_hash<T: AsRef<[u8]>>(data: T) -> Hash {
    Hash(Blake2b::<U32>::digest(data).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn test_blake2b_hash() {
        let data = "Hello Radix";
        let hash = blake2b_256_hash(data);
        assert_eq!(
            hash,
            Hash::from_str("48f1bd08444b5e713db9e14caac2faae71836786ac94d645b00679728202a935")
                .unwrap()
        );
    }
}
