use arbitrary::{Arbitrary, Unstructured};
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::{FromPublicKey, NonFungibleGlobalId};
use radix_engine_interface::data::manifest::manifest_decode;
use scrypto_unit::{TestRunner, TestRunnerSnapshot};
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::manifest::ast;
use transaction::model::Instruction;
use transaction::model::TransactionManifest;

const INSTRUCTION_MAX_CNT: u8 = 3;

#[derive(Debug, Clone)]
struct Account {
    public_key: EcdsaSecp256k1PublicKey,
    //_private_key: EcdsaSecp256k1PrivateKey,
    #[allow(unused)]
    address: ComponentAddress,
    #[allow(unused)]
    resources: Vec<ResourceAddress>,
}

pub struct TxFuzzer {
    runner: TestRunner,
    snapshot: TestRunnerSnapshot,
    accounts: Vec<Account>,
}

impl TxFuzzer {
    pub fn new() -> Self {
        let mut runner = TestRunner::builder().without_trace().build();
        let accounts: Vec<Account> = (0..2)
            .map(|_| {
                let acc = runner.new_account(false);
                let resources: Vec<ResourceAddress> = vec![
                    runner.create_fungible_resource(10000.into(), 18, acc.2),
                    runner.create_fungible_resource(10000.into(), 18, acc.2),
                    runner.create_non_fungible_resource(acc.2),
                    runner.create_non_fungible_resource(acc.2),
                ];
                println!("addr = {:?}", acc.2);
                Account {
                    public_key: acc.0,
                    //_private_key: acc.1,
                    address: acc.2,
                    resources,
                }
            })
            .collect();
        let snapshot = runner.create_snapshot();

        Self {
            runner,
            snapshot,
            accounts,
        }
    }

    pub fn reset_runner(&mut self) {
        self.runner.restore_snapshot(self.snapshot.clone());
    }

    // pick account from the preallocated pool basing on the input data
    #[cfg(feature = "smart_mutate")]
    fn get_account(&mut self, data: &[u8]) -> Option<ComponentAddress> {
        let len = data.len();
        if len >= 2 && data[len - 2] % 2 == 0 {
            let idx = *data.last().unwrap() as usize % self.accounts.len();
            return Some(self.accounts[idx].address);
        }
        None
    }

    // pick resource from the preallocated pool basing on the input data
    #[cfg(feature = "smart_mutate")]
    fn get_resource(&mut self, data: &[u8]) -> Option<ResourceAddress> {
        let len = data.len();
        if len >= 3 && data[len - 3] % 2 == 0 {
            let account_idx = *data.last().unwrap() as usize % self.accounts.len();
            let resource_idx = data[len - 2] as usize % self.accounts[account_idx].resources.len();
            Some(self.accounts[account_idx].resources[resource_idx])
        } else {
            None
        }
    }

    // Smartly replace some data in the manifest using some preallocated resources.
    // This is to let fuzzing go "deeper" into the manifest instructions and not to reject the
    // transaction on the very early stage
    #[cfg(feature = "smart_mutate")]
    fn smart_mutate_manifest(&mut self, manifest: &mut TransactionManifest) {
        for i in &mut manifest.instructions {
            match i {
                Instruction::CallMethod {
                    component_address, ..
                }
                | Instruction::SetComponentRoyaltyConfig {
                    component_address, ..
                }
                | Instruction::ClaimComponentRoyalty { component_address } => {
                    if let Some(address) = self.get_account(component_address.as_ref()) {
                        *component_address = address;
                    }
                }
                Instruction::TakeFromWorktop { resource_address }
                | Instruction::TakeFromWorktopByAmount {
                    resource_address, ..
                }
                | Instruction::TakeFromWorktopByIds {
                    resource_address, ..
                }
                | Instruction::AssertWorktopContains { resource_address }
                | Instruction::AssertWorktopContainsByAmount {
                    resource_address, ..
                }
                | Instruction::AssertWorktopContainsByIds {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZone { resource_address }
                | Instruction::CreateProofFromAuthZoneByAmount {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZoneByIds {
                    resource_address, ..
                }
                | Instruction::MintFungible {
                    resource_address, ..
                }
                | Instruction::MintNonFungible {
                    resource_address, ..
                }
                | Instruction::MintUuidNonFungible {
                    resource_address, ..
                } => {
                    if let Some(address) = self.get_resource(resource_address.as_ref()) {
                        *resource_address = address;
                    }
                }
                _ => {}
            }
        }
    }

    fn get_random_account(
        &mut self,
        unstructured: &mut Unstructured,
    ) -> (Account, ResourceAddress) {
        let account_idx = unstructured.choose_index(self.accounts.len()).unwrap();
        let mut account = self.accounts[account_idx].clone();

        let resource_idx = unstructured.choose_index(account.resources.len()).unwrap();
        let mut resource_address = account.resources[resource_idx];

        (account, resource_address)
    }

    fn build_manifest(&mut self, data: &[u8]) -> Result<TransactionManifest, TxStatus> {
        let mut builder = ManifestBuilder::new();
        let mut buckets: Vec<ManifestBucket> = vec![];
        let mut proof_ids: Vec<ManifestProof> = vec![];

        let mut unstructured = Unstructured::new(&data);
        println!("unstructured init len = {}", unstructured.len());

        let (account, resource_address) = self.get_random_account(&mut unstructured);
        let component_address = account.address;

        let fee = Decimal::from(100);
        //builder.lock_fee(self.runner.faucet_component(), fee);
        builder.lock_fee(component_address, fee);

        let mut i = 0;
        while i < INSTRUCTION_MAX_CNT && unstructured.len() > 0 {
            println!("unstructured remaining len = {}", unstructured.len());
            let next: u8 = unstructured.int_in_range(0..=1).unwrap();
            println!(
                "unstructured remaining len = {} next = {}",
                unstructured.len(),
                next
            );

            let instruction = match next {
                // AssertWorktopContains
                0 => Some(Instruction::AssertWorktopContains { resource_address }),
                1 => {
                    let amount: u128 = unstructured.int_in_range(0..=10_000_000u128).unwrap();
                    let amount = Decimal::from(amount);
                    Some(Instruction::AssertWorktopContainsByAmount {
                        amount,
                        resource_address,
                    })
                }
                2 => None,
                _ => unreachable!(
                    "Not all instructions (current count is {}) covered by this match",
                    2
                ),
            };
            match instruction {
                Some(instruction) => {
                    let (_, bucket_id, proof_id) = builder.add_instruction(instruction);
                    match bucket_id {
                        Some(bucket_id) => buckets.push(bucket_id),
                        None => {}
                    }
                    match proof_id {
                        Some(proof_id) => proof_ids.push(proof_id),
                        None => {}
                    }
                    i += 1;
                }
                None => {}
            }
        }

        let manifest = builder.build();
        println!("manifest = {:?}", manifest);
        Ok(manifest)
    }

    pub fn fuzz_tx_manifest(&mut self, data: &[u8]) -> TxStatus {
        //        let result = manifest_decode::<TransactionManifest>(data);
        let result = self.build_manifest(data);
        match result {
            #[allow(unused_mut)]
            Ok(mut manifest) => {
                #[cfg(feature = "smart_mutate")]
                self.smart_mutate_manifest(&mut manifest);

                let receipt = self.runner.execute_manifest(
                    manifest,
                    vec![NonFungibleGlobalId::from_public_key(
                        &self.accounts[0].public_key,
                    )],
                );
                if receipt.is_commit_success() {
                    TxStatus::CommitSuccess
                } else {
                    TxStatus::CommitFailure
                }
            }
            Err(_err) => TxStatus::DecodeError,
        }
    }
}

#[derive(Debug)]
pub enum TxStatus {
    // Transaction commit success
    CommitSuccess,
    // Transaction commit failure
    CommitFailure,
    // Transaction manifest parse error
    DecodeError,
}

#[test]
// This test verifies whether it is still possible to parse manifest raw files and execute them.
// If it fails with TxStatus::DecodeError then most likely that manifest format has changed and
// input files shall be recreated.
fn test_fuzz_tx() {
    let mut fuzzer = TxFuzzer::new();
    let data = std::fs::read(
        "fuzz_input/transaction/manifest_01995e0d6005c34ad99fba993ebe1443ef55c4db71ed037de12afb3eb28bbfae.raw",
    )
    .unwrap();
    assert!(matches!(
        fuzzer.fuzz_tx_manifest(&data),
        TxStatus::CommitSuccess
    ));

    let data = std::fs::read(
        "fuzz_input/transaction//manifest_0113970c0a72935c8c27ddd97a9396d1839f0173bf9ed091f9706aa61db8417e.raw",
    )
    .unwrap();
    assert!(matches!(
        fuzzer.fuzz_tx_manifest(&data),
        TxStatus::CommitFailure
    ));
}

// Initialize static objects outside the fuzzing loop to assure deterministic instrumentation
// output across runs.
pub fn fuzz_tx_init_statics() {
    // Following code initializes secp256k1::SECP256K1 global static context
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(100).unwrap();
    let _public_key = private_key.public_key();
}
