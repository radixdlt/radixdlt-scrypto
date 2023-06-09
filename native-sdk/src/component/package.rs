use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::data::scrypto::{scrypto_encode, ScryptoDecode};
use radix_engine_interface::types::PackageAddress;
use radix_engine_interface::types::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::ToString;

#[derive(Debug)]
pub struct BorrowedPackage(pub PackageAddress);
