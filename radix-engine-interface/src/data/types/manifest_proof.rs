use crate::scrypto;
use sbor::*;

#[scrypto(TypeId, Encode, Decode)]
pub struct ManifestProof(u32);
