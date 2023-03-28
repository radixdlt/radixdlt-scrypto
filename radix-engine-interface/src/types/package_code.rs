use sbor::rust::vec::Vec;

pub enum PackageCode {
    Wasm(Vec<u8>),
    Native,
}
