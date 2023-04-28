use arbitrary::{Arbitrary, Unstructured};
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::resource::{FromPublicKey, NonFungibleGlobalId};
#[cfg(feature = "decode_tx_manifest")]
use radix_engine_interface::data::manifest::manifest_decode;
use radix_engine_stores::interface::SubstateDatabase;
use radix_engine_stores::jmt_support::JmtMapper;
use scrypto_unit::{TestRunner, TestRunnerSnapshot};
use strum::EnumCount;
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
    component_addresses: Vec<ComponentAddress>,
    all_resource_addresses: Vec<ResourceAddress>,
    fungible_resource_addresses: Vec<ResourceAddress>,
    non_fungible_resource_addresses: Vec<ResourceAddress>,
    #[allow(unused)]
    package_addresses: Vec<PackageAddress>,
}

impl TxFuzzer {
    pub fn new() -> Self {
        let mut runner = TestRunner::builder().without_trace().build();
        let mut component_addresses = vec![runner.faucet_component()];
        let mut all_resource_addresses = vec![
            RADIX_TOKEN,
            ECDSA_SECP256K1_TOKEN,
            EDDSA_ED25519_TOKEN,
            SYSTEM_TOKEN,
            PACKAGE_TOKEN,
            GLOBAL_OBJECT_TOKEN,
            PACKAGE_OWNER_TOKEN,
            VALIDATOR_OWNER_TOKEN,
            IDENTITY_OWNER_TOKEN,
            ACCOUNT_OWNER_TOKEN,
        ];
        let mut non_fungible_resource_addresses = vec![];
        let mut fungible_resource_addresses = vec![];
        let package_addresses = vec![
            PACKAGE_PACKAGE,
            RESOURCE_MANAGER_PACKAGE,
            IDENTITY_PACKAGE,
            EPOCH_MANAGER_PACKAGE,
            CLOCK_PACKAGE,
            ACCOUNT_PACKAGE,
            ACCESS_CONTROLLER_PACKAGE,
            TRANSACTION_PROCESSOR_PACKAGE,
            METADATA_PACKAGE,
            ROYALTY_PACKAGE,
            ACCESS_RULES_PACKAGE,
            GENESIS_HELPER_PACKAGE,
        ];
        let accounts: Vec<Account> = (0..2)
            .map(|_| {
                let acc = runner.new_account(false);
                let resources: Vec<ResourceAddress> = vec![
                    runner.create_fungible_resource(10000.into(), 18, acc.2),
                    runner.create_fungible_resource(10000.into(), 18, acc.2),
                    runner.create_non_fungible_resource(acc.2),
                    runner.create_non_fungible_resource(acc.2),
                ];
                all_resource_addresses.append(&mut resources.clone());
                fungible_resource_addresses.append(&mut resources.clone()[0..2].to_vec());
                non_fungible_resource_addresses.append(&mut resources.clone()[2..4].to_vec());
                println!("addr = {:?}", acc.2);
                component_addresses.push(acc.2);

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
            component_addresses,
            all_resource_addresses,
            fungible_resource_addresses,
            non_fungible_resource_addresses,
            package_addresses,
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

    #[allow(unused)]
    fn get_random_account(
        &mut self,
        unstructured: &mut Unstructured,
    ) -> Result<(Account, ResourceAddress), arbitrary::Error> {
        let account = unstructured.choose(&self.accounts[..])?;
        let resource_address = unstructured.choose(&account.resources[..])?;

        Ok((account.clone(), resource_address.clone()))
    }

    fn get_non_fungible_local_id(
        &mut self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> BTreeSet<NonFungibleLocalId> {
        let vaults = self
            .runner
            .get_component_vaults(component_address, resource_address);
        let mut btree_ids = btreeset![];
        for vault in vaults {
            let vault = self
                .runner
                .substate_db()
                .get_mapped_substate::<JmtMapper, LiquidNonFungibleVault>(
                    &vault,
                    SysModuleId::Object.into(),
                    NonFungibleVaultOffset::LiquidNonFungible.into(),
                )
                .map(|vault| vault.ids);

            vault.map(|ids| {
                let mut substate_iter = self
                    .runner
                    .substate_db()
                    .list_mapped_substates::<JmtMapper>(
                        ids.as_node_id(),
                        SysModuleId::Object.into(),
                    );
                substate_iter.next().map(|(_key, value)| {
                    let id: NonFungibleLocalId = scrypto_decode(value.as_slice()).unwrap();
                    btree_ids.insert(id);
                });
            });
        }
        btree_ids
    }

    fn build_manifest(&mut self, data: &[u8]) -> Result<TransactionManifest, TxStatus> {
        let mut builder = ManifestBuilder::new();
        let mut buckets: Vec<ManifestBucket> = vec![];
        let mut proof_ids: Vec<ManifestProof> = vec![];

        // Arbitrary does not return error if not enough data to construct a full instance of
        // Self. It uses dummy values (zeros) instead.
        // TODO: to consider if this is ok to allow it.

        let mut unstructured = Unstructured::new(&data);
        println!("unstructured init len = {}", unstructured.len());

        let resource_address = unstructured
            .choose(&self.all_resource_addresses[..])
            .unwrap()
            .clone();
        let component_address = unstructured
            .choose(&self.component_addresses[..])
            .unwrap()
            .clone();
        let package_address = unstructured
            .choose(&self.package_addresses[..])
            .unwrap()
            .clone();
        let non_fungible_resource_address = unstructured
            .choose(&self.non_fungible_resource_addresses[..])
            .unwrap()
            .clone();
        // TODO: if resource_address of not NonFungible resource is given then we got panic in get_mapped_substate
        // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: UnexpectedSize { expected: 2, actual: 1 }', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-stores/src/interface.rs:200:41
        let non_fungible_ids =
            self.get_non_fungible_local_id(component_address, non_fungible_resource_address);

        let fee = Decimal::arbitrary(&mut unstructured).unwrap();
        builder.lock_fee(component_address, fee);

        let mut i = 0;
        while i < INSTRUCTION_MAX_CNT && unstructured.len() > 0 {
            println!("unstructured remaining len = {}", unstructured.len());
            let next: u8 = unstructured
                .int_in_range(0..=ast::Instruction::COUNT as u8 - 1)
                .unwrap();
            println!(
                "unstructured remaining len = {} next = {}",
                unstructured.len(),
                next
            );

            let instruction = match next {
                // AssertWorktopContains
                0 => Some(Instruction::AssertWorktopContains { resource_address }),
                // AssertWorktopContainsByAmount
                1 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::AssertWorktopContainsByAmount {
                        amount,
                        resource_address,
                    })
                }
                // AssertWorktopContainsByIds
                2 => Some(Instruction::AssertWorktopContainsByIds {
                    ids: non_fungible_ids.clone(),
                    resource_address,
                }),
                // BurnResource
                3 => {
                    let bucket_id = match unstructured.choose(&buckets[..]) {
                        Ok(bucket_id) => *bucket_id,
                        Err(_) => ManifestBucket::arbitrary(&mut unstructured).unwrap(),
                    };

                    Some(Instruction::BurnResource { bucket_id })
                }
                // CallFunction
                4 => {
                    // TODO
                    None
                }
                // CallMethod
                5 => {
                    // TODO
                    None
                }
                // ClaimComponentRoyalty
                6 => Some(Instruction::ClaimComponentRoyalty { component_address }),
                // ClaimPackageRoyalty
                7 => Some(Instruction::ClaimPackageRoyalty { package_address }),
                // ClearAuthZone
                8 => Some(Instruction::ClearAuthZone),
                // ClearSignatureProofs
                9 => Some(Instruction::ClearSignatureProofs),
                // CloneProof
                10 => {
                    let proof_id = match unstructured.choose(&proof_ids[..]) {
                        Ok(proof_id) => *proof_id,
                        Err(_) => ManifestProof::arbitrary(&mut unstructured).unwrap(),
                    };
                    Some(Instruction::CloneProof { proof_id })
                }

                11..=43 => None,
                _ => unreachable!(
                    "Not all instructions (current count is {}) covered by this match",
                    ast::Instruction::COUNT
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
        #[cfg(feature = "decode_tx_manifest")]
        let result = manifest_decode::<TransactionManifest>(data);
        #[cfg(not(feature = "decode_tx_manifest"))]
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
    // Transaction manifest build error
    #[allow(unused)]
    ManifestBuildError,
    // Transaction commit success
    CommitSuccess,
    // Transaction commit failure
    CommitFailure,
    // Transaction manifest parse error
    #[allow(unused)]
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
