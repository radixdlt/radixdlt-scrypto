use sbor::*;
use scrypto::buffer::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

/// A in-memory version of Radix Engine.
pub struct InMemoryRadixEngine {
    ledger: InMemoryLedger,
    nonce: u32,
    verbose: bool,
}

impl InMemoryRadixEngine {
    /// Creates a new in-memory radix engine.
    pub fn new(verbose: bool) -> Self {
        Self {
            ledger: InMemoryLedger::new(),
            nonce: 0,
            verbose,
        }
    }

    /// Publishes a package.
    pub fn publish(&mut self, code: &[u8]) -> Result<Address, RuntimeError> {
        let tx_hash = self.next_tx_hash();
        let mut runtime = Runtime::new(tx_hash, &mut self.ledger);

        let address = runtime.new_package_address();
        validate_module(code)?;
        runtime.put_package(address, Package::new(code.to_owned()));
        runtime.flush();

        Ok(address)
    }

    /// Publishes a package at a specific address.
    pub fn publish_at(&mut self, code: &[u8], address: Address) -> Result<Address, RuntimeError> {
        let tx_hash = self.next_tx_hash();
        let mut runtime = Runtime::new(tx_hash, &mut self.ledger);

        validate_module(code)?;
        runtime.put_package(address, Package::new(code.to_owned()));
        runtime.flush();

        Ok(address)
    }

    /// Calls a function.
    pub fn call_function<T: Decode>(
        &mut self,
        package: Address,
        blueprint: &str,
        function: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<T, RuntimeError> {
        let tx_hash = self.next_tx_hash();
        let mut runtime = Runtime::new(tx_hash, &mut self.ledger);
        let mut process = Process::new(0, self.verbose, &mut runtime);
        let target =
            process.prepare_call_function(package, blueprint, function.to_owned(), args)?;
        let result = process.run(target);
        process.finalize()?;
        match result {
            Ok(bytes) => {
                runtime.flush();
                Ok(scrypto_decode(&bytes).map_err(|e| RuntimeError::InvalidData(e))?)
            }
            Err(e) => Err(e),
        }
    }

    /// Calls a method.
    pub fn call_method<T: Decode>(
        &mut self,
        component: Address,
        method: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<T, RuntimeError> {
        let tx_hash = self.next_tx_hash();
        let mut runtime = Runtime::new(tx_hash, &mut self.ledger);
        let mut process = Process::new(0, self.verbose, &mut runtime);
        let target = process.prepare_call_method(component, method.to_owned(), args)?;
        let result = process.run(target);
        process.finalize()?;
        match result {
            Ok(bytes) => {
                runtime.flush();
                Ok(scrypto_decode(&bytes).map_err(|e| RuntimeError::InvalidData(e))?)
            }
            Err(e) => Err(e),
        }
    }

    fn next_tx_hash(&mut self) -> H256 {
        let tx_hash = sha256(self.nonce.to_string());
        self.nonce += 1;
        tx_hash
    }
}
