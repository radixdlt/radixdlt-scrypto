use scrypto::abi;
use scrypto::types::*;

use crate::engine::RuntimeError;

pub trait AbiProvider {
    /// Exports the ABI of a blueprint.
    fn export_abi<A: AsRef<str>>(
        &self,
        package: Address,
        name: A,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError>;

    /// Exports the ABI of the blueprint from which the given component is instantiated.
    fn export_abi_component(
        &self,
        component: Address,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError>;
}

pub struct MockAbiProvider {}
