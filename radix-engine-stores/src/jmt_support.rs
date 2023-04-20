use crate::interface::SubstateKeyMapper;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::SubstateKey;

pub struct JmtKeyMapper;

impl SubstateKeyMapper for JmtKeyMapper {
    fn map_to_db_key(key: SubstateKey) -> Vec<u8> {
        let bytes = match key {
            SubstateKey::Key(key) => {
                // TODO: Split into Map/Tuple
                if key.len() == 1 {
                    key
                } else {
                    hash(key).0[12..32].to_vec()
                }
            }
            SubstateKey::Sorted(bucket, key) => {
                let mut bytes = bucket.to_be_bytes().to_vec();
                bytes.extend(hash(key).0[12..32].to_vec()); // 20 bytes
                bytes
            }
        };

        bytes
    }
}
