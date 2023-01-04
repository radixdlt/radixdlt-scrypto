use crate::scrypto;
use sbor::*;

#[scrypto(TypeId, Encode, Decode)]
pub struct ManifestBucket(u32);
