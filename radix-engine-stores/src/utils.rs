pub fn encode_substate_id(index_id: &Vec<u8>, db_key: &Vec<u8>) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(index_id);
    buffer.extend(db_key); // Length is marked by EOF
    buffer
}

// TODO: Clean this interface up and move size of hash to a more appropriate interface
pub fn decode_substate_id(slice: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if slice.len() >= 26 {
        let index_id = slice[0..26].to_vec();
        let key = slice[26 + 1..].to_vec();

        return Some((index_id, key));
    }

    return None;
}
