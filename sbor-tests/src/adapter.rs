use bincode::{config, Decode, Encode};
use serde::{Deserialize, Serialize};

pub fn bincode_encode<T: Encode>(v: &T) -> Vec<u8> {
    let config = config::standard();
    bincode::encode_to_vec(v, config).unwrap()
}

pub fn bincode_decode<T: Decode>(buf: &[u8]) -> Result<T, String> {
    let config = config::standard();
    let (decoded, _len): (T, usize) = bincode::decode_from_slice(buf, config).unwrap();
    Ok(decoded)
}

pub fn json_encode<T: Serialize>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}

pub fn json_decode<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> Result<T, String> {
    serde_json::from_slice(buf).map_err(|e| e.to_string())
}
