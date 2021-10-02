use scrypto::abi;
use scrypto::buffer::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

/// An interface for exporting the ABI of a blueprint.
pub trait AbiProvider {
    /// Exports the ABI of a blueprint.
    fn export_abi<S: AsRef<str>>(
        &self,
        package: Address,
        name: S,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError>;

    /// Exports the ABI of the blueprint, from which the given component is instantiated.
    fn export_abi_component(
        &self,
        component: Address,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError>;
}

/// This basic ABI provider can provide ABIs of bootstrapped and manually
/// added blueprints.
pub struct BasicAbiProvider {
    ledger: InMemoryLedger,
}

impl BasicAbiProvider {
    pub fn new() -> Self {
        Self {
            ledger: InMemoryLedger::with_bootstrap(),
        }
    }

    pub fn with_package(&mut self, address: Address, code: Vec<u8>) -> &mut Self {
        self.ledger.put_package(address, Package::new(code));
        self
    }

    pub fn with_component(
        &mut self,
        address: Address,
        package: Address,
        name: String,
        state: Vec<u8>,
    ) -> &mut Self {
        self.ledger
            .put_component(address, Component::new(package, name, state));
        self
    }
}

impl AbiProvider for BasicAbiProvider {
    fn export_abi<S: AsRef<str>>(
        &self,
        package: Address,
        name: S,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        // Deterministic transaction context
        let mut ledger = self.ledger.clone();
        let current_epoch = 0;
        let tx_hash = sha256([]);

        // Start a process and run abi generator
        let mut track = Track::new(&mut ledger, current_epoch, tx_hash);
        let mut proc = track.start_process(trace);
        let output: (Vec<abi::Function>, Vec<abi::Method>) = proc
            .call_abi(package, name.as_ref())
            .and_then(|rtn| scrypto_decode(&rtn).map_err(RuntimeError::InvalidData))?;

        // Return ABI
        Ok(abi::Blueprint {
            package: package.to_string(),
            name: name.as_ref().to_string(),
            functions: output.0,
            methods: output.1,
        })
    }

    fn export_abi_component(
        &self,
        component: Address,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let c = self
            .ledger
            .get_component(component)
            .ok_or(RuntimeError::ComponentNotFound(component))?;
        self.export_abi(c.package(), c.name().to_owned(), trace)
    }
}
