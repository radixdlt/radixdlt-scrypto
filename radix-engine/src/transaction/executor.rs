use scrypto::abi;
use scrypto::args;
use scrypto::buffer::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;

use crate::engine::*;
use crate::ledger::*;
use crate::transaction::*;

/// A transaction executor.
pub struct TransactionExecutor<'l, L: Ledger> {
    ledger: &'l mut L,
    epoch: u64,
    nonce: u64,
}

impl<'l, L: Ledger> TransactionExecutor<'l, L> {
    pub fn new(ledger: &'l mut L, epoch: u64, nonce: u64) -> Self {
        Self {
            ledger,
            epoch,
            nonce,
        }
    }

    pub fn set_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
    }

    pub fn new_account(&mut self, trace: bool) -> Address {
        self.ledger.bootstrap();
        let abi = self.export_abi(ACCOUNT_PACKAGE, "Account", false).unwrap();

        let transaction = TransactionBuilder::new()
            .call_function(&abi, "new", vec![])
            .build_with(None)
            .unwrap();

        let receipt = self.execute(&transaction, trace);
        receipt.nth_component(0).unwrap()
    }

    pub fn publish_package(&mut self, code: &[u8], trace: bool) -> Address {
        self.ledger.bootstrap();
        let abi = self.export_abi(SYSTEM_PACKAGE, "System", false).unwrap();

        let transaction = TransactionBuilder::new()
            .instruction(Instruction::CallFunction {
                blueprint: (abi.package.parse().unwrap(), abi.name.to_string()),
                function: "publish_package".to_string(),
                args: vec![scrypto_encode(code)],
            })
            .build_with(None)
            .unwrap();

        let receipt = self.execute(&transaction, trace);
        receipt.nth_package(0).unwrap()
    }

    pub fn new_resource_mutable(
        &mut self,
        metadata: HashMap<String, String>,
        minter: Address, trace: bool
    ) -> Address {
        self.ledger.bootstrap();
        let abi = self.export_abi(SYSTEM_PACKAGE, "System", false).unwrap();

        let transaction = TransactionBuilder::new()
            .instruction(Instruction::CallFunction {
                blueprint: (abi.package.parse().unwrap(), abi.name.to_string()),
                function: "new_resource_mutable".to_string(),
                args: vec![scrypto_encode(&metadata), scrypto_encode(&minter)],
            })
            .build_with(None)
            .unwrap();

        let receipt = self.execute(&transaction, trace);
        receipt.nth_resource_def(0).unwrap()
    }

    pub fn new_resource_fixed(
        &mut self,
        metadata: HashMap<String, String>,
        supply: Amount,
        recipient: Address, trace: bool
    ) -> Address {
        self.ledger.bootstrap();
        let abi = self.export_abi(SYSTEM_PACKAGE, "System", false).unwrap();

        let transaction = TransactionBuilder::new()
            .instruction(Instruction::CallFunction {
                blueprint: (abi.package.parse().unwrap(), abi.name.to_string()),
                function: "new_resource_fixed".to_string(),
                args: vec![scrypto_encode(&metadata), scrypto_encode(&supply)],
            })
            .build_with(Some(recipient))
            .unwrap();

        let receipt = self.execute(&transaction, trace);
        receipt.nth_resource_def(0).unwrap()
    }

    pub fn mint_resource(&mut self, amount: Amount, resource_address: Address, recipient: Address, trace: bool) {
        self.ledger.bootstrap();
        let abi = self.export_abi(SYSTEM_PACKAGE, "System", false).unwrap();

        let transaction = TransactionBuilder::new()
            .instruction(Instruction::CallFunction {
                blueprint: (abi.package.parse().unwrap(), abi.name.to_string()),
                function: "mint_resource".to_string(),
                args: vec![scrypto_encode(&amount), scrypto_encode(&resource_address)],
            })
            .build_with(Some(recipient))
            .unwrap();

        self.execute(&transaction, trace);
    }

    pub fn export_abi_by_component(
        &self,
        component: Address,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let c = self
            .ledger
            .get_component(component)
            .ok_or(RuntimeError::ComponentNotFound(component))?;
        self.export_abi(c.blueprint().0.clone(), c.blueprint().1.as_ref(), trace)
    }

    pub fn export_abi(
        &self,
        package: Address,
        blueprint: &str,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        // deterministic ledger, epoch and transaction hash
        let mut ledger = InMemoryLedger::new();
        ledger.put_package(
            package,
            self.ledger
                .get_package(package)
                .ok_or(RuntimeError::PackageNotFound(package.to_owned()))?,
        );
        let epoch = 0;
        let tx_hash = sha256([]);

        // Start a process and run abi generator
        let mut track = Track::new(&mut ledger, epoch, tx_hash);
        let mut proc = track.start_process(trace);
        let output: (Vec<abi::Function>, Vec<abi::Method>) = proc
            .call_abi((package, blueprint.to_owned()))
            .and_then(|rtn| scrypto_decode(&rtn).map_err(RuntimeError::InvalidData))?;

        Ok(abi::Blueprint {
            package: package.to_string(),
            name: blueprint.to_string(),
            functions: output.0,
            methods: output.1,
        })
    }

    pub fn execute(&mut self, tx: &Transaction, trace: bool) -> Receipt {
        let mut track = Track::new(self.ledger, self.epoch, sha256(self.nonce.to_string()));
        let mut proc = track.start_process(trace);

        let mut results = vec![];
        let mut success = true;
        for inst in &tx.instructions {
            let res = match inst {
                Instruction::ReserveBucket => {
                    // TODO check if this is the first instruction
                    proc.reserve_bucket();
                    Ok(None)
                }
                Instruction::BorrowBucket { bid } => {
                    // TODO check if this is the first instruction
                    proc.borrow_bucket(*bid).map(|_| None)
                }
                Instruction::MoveToBucket {
                    amount,
                    resource_address,
                    bid,
                } => proc
                    .move_to_bucket(*amount, *resource_address, *bid)
                    .map(|_| None),
                Instruction::CallFunction {
                    blueprint,
                    function,
                    args,
                } => proc
                    .call_function(blueprint.clone(), function.as_str(), args.clone())
                    .map(Option::from),
                Instruction::CallMethod {
                    component,
                    method,
                    args,
                } => proc
                    .call_method(*component, method.as_str(), args.clone())
                    .map(Option::from),
                Instruction::DepositAll { component, method } => {
                    let buckets: Vec<_> = proc
                        .owned_buckets()
                        .iter()
                        .map(|bid| scrypto::resource::Bucket::from(*bid))
                        .collect();
                    if !buckets.is_empty() {
                        proc.call_method(*component, method.as_str(), args!(buckets))
                            .map(Option::from)
                    } else {
                        Ok(None)
                    }
                }
                Instruction::Finalize => proc.finalize().map(|_| None),
            };
            results.push(res);
            if results.last().unwrap().is_err() {
                success = false;
                break;
            }
        }

        // commit state updates
        if success {
            track.commit();
            self.nonce += 1;
        }

        Receipt {
            transaction: tx.clone(),
            success,
            results,
            logs: track.logs().clone(),
            new_addresses: track.new_addresses().to_vec(),
        }
    }
}
