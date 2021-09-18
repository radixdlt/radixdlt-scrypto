use sbor::*;
use scrypto::buffer::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::HashMap;
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
    prepared_buckets: HashMap<BID, Bucket>, // prepared for next invocation
    prepared_references: HashMap<RID, BucketRef>, // prepared for next invocation
    alloc: AddressAllocator,
}

impl InMemoryRadixEngine {
    /// Creates a new in-memory radix engine.
    pub fn new(verbose: bool) -> Self {
        Self {
            ledger: InMemoryLedger::new(),
            nonce: 0,
            verbose,
            prepared_buckets: HashMap::new(),
            prepared_references: HashMap::new(),
            alloc: AddressAllocator::new(),
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

    /// Prepare bucket for next invocation.
    pub fn prepare_bucket(&mut self, amount: Amount, resource: Address) -> BID {
        let bid = self.alloc.new_bucket_id();
        self.prepared_buckets
            .insert(bid, Bucket::new(amount, resource));
        bid
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
        let invocation =
            process.prepare_call_function(package, blueprint, function.to_owned(), args)?;

        // move resources
        process.put_resources(
            self.prepared_buckets.drain().collect(),
            self.prepared_references.drain().collect(),
        );
        self.alloc.reset();

        // run
        let result = process.run(invocation);
        process.finalize()?;

        // check
        match result {
            Ok(bytes) => {
                runtime.flush();
                Ok(scrypto_decode(&bytes).map_err(RuntimeError::InvalidData)?)
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
        let invocation = process.prepare_call_method(component, method.to_owned(), args)?;

        // move resources
        process.put_resources(
            self.prepared_buckets.drain().collect(),
            self.prepared_references.drain().collect(),
        );
        self.alloc.reset();

        // run
        let result = process.run(invocation);
        process.finalize()?;

        // check
        match result {
            Ok(bytes) => {
                runtime.flush();
                Ok(scrypto_decode(&bytes).map_err(RuntimeError::InvalidData)?)
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
