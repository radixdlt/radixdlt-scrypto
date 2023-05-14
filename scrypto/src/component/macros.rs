use crate::component::*;
use crate::engine::scrypto_env::ScryptoEnv;
use radix_engine_interface::api::*;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::scrypto_encode;
use sbor::rust::prelude::*;

#[macro_export]
macro_rules! borrow_package {
    ($address:expr) => {
        $crate::component::BorrowedPackage($address.clone())
    };
}
