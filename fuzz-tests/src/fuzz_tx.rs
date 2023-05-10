use arbitrary::{Arbitrary, Unstructured};
use radix_engine::track::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::{FromPublicKey, NonFungibleGlobalId};
#[cfg(feature = "dummy_fuzzing")]
use radix_engine_interface::data::manifest::manifest_decode;
use scrypto_unit::{TestRunner, TestRunnerSnapshot};
#[cfg(test)]
use std::panic::{catch_unwind, AssertUnwindSafe};
use strum::EnumCount;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::manifest::ast;
use transaction::model::Instruction;
use transaction::model::TransactionManifest;

#[allow(unused)]
const INSTRUCTION_MAX_CNT: u8 = 10;

// Verbose version
#[cfg(feature = "verbose")]
macro_rules! dbg {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

// Non-verbose version
#[cfg(not(feature = "verbose"))]
macro_rules! dbg {
    ($( $args:expr ),*) => {};
}

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
    #[allow(unused)]
    component_addresses: Vec<ComponentAddress>,
    #[allow(unused)]
    all_resource_addresses: Vec<ResourceAddress>,
    #[allow(unused)]
    fungible_resource_addresses: Vec<ResourceAddress>,
    #[allow(unused)]
    non_fungible_resource_addresses: Vec<ResourceAddress>,
    package_addresses: Vec<PackageAddress>,
    public_keys: Vec<EcdsaSecp256k1PublicKey>,
}

impl TxFuzzer {
    pub fn new() -> Self {
        let mut runner = TestRunner::builder().without_trace().build();
        let mut component_addresses = vec![EPOCH_MANAGER, CLOCK, GENESIS_HELPER, FAUCET];
        let mut all_resource_addresses = vec![
            RADIX_TOKEN,
            ECDSA_SECP256K1_SIGNATURE_VIRTUAL_BADGE,
            EDDSA_ED25519_SIGNATURE_VIRTUAL_BADGE,
            SYSTEM_TRANSACTION_BADGE,
            PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE,
            GLOBAL_CALLER_VIRTUAL_BADGE,
            PACKAGE_OWNER_BADGE,
            VALIDATOR_OWNER_BADGE,
            IDENTITY_OWNER_BADGE,
            ACCOUNT_OWNER_BADGE,
        ];
        let mut non_fungible_resource_addresses = vec![];
        let mut fungible_resource_addresses = vec![];
        let package_addresses = vec![
            PACKAGE_PACKAGE,
            RESOURCE_PACKAGE,
            IDENTITY_PACKAGE,
            EPOCH_MANAGER_PACKAGE,
            CLOCK_PACKAGE,
            ACCOUNT_PACKAGE,
            ACCESS_CONTROLLER_PACKAGE,
            TRANSACTION_PROCESSOR_PACKAGE,
            METADATA_MODULE_PACKAGE,
            ROYALTY_MODULE_PACKAGE,
            ACCESS_CONTROLLER_PACKAGE,
            GENESIS_HELPER_PACKAGE,
            FAUCET_PACKAGE,
        ];
        let mut public_keys = vec![];
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
                component_addresses.push(acc.2);
                public_keys.push(acc.0);

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
            public_keys,
        }
    }

    pub fn reset_runner(&mut self) {
        self.runner.restore_snapshot(self.snapshot.clone());
    }

    #[allow(unused)]
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
            let mut substate_iter = self
                .runner
                .substate_db()
                .list_mapped::<SpreadPrefixKeyMapper, NonFungibleLocalId, MapKey>(
                    &vault,
                    OBJECT_BASE_PARTITION
                        .at_offset(PartitionOffset(1u8))
                        .unwrap(),
                );

            substate_iter.next().map(|(_key, id)| {
                btree_ids.insert(id);
            });
        }
        btree_ids
    }

    #[allow(unused)]
    fn build_manifest(&mut self, data: &[u8]) -> Result<TransactionManifest, TxStatus> {
        // Arbitrary does not return error if not enough data to construct a full instance of
        // Self. It uses dummy values (zeros) instead.
        // TODO: to consider if this is ok to allow it.
        let mut unstructured = Unstructured::new(&data);

        let mut builder = ManifestBuilder::new();
        let mut buckets: Vec<ManifestBucket> =
            vec![ManifestBucket::arbitrary(&mut unstructured).unwrap()];
        let mut proof_ids: Vec<ManifestProof> =
            vec![ManifestProof::arbitrary(&mut unstructured).unwrap()];

        let mut public_keys = self.public_keys.clone();
        public_keys.push(EcdsaSecp256k1PublicKey::arbitrary(&mut unstructured).unwrap());

        let public_key = unstructured.choose(&public_keys[..]).unwrap().clone();

        let mut package_addresses = self.package_addresses.clone();

        let resource_address = unstructured
            .choose(&self.all_resource_addresses[..])
            .unwrap()
            .clone();
        let component_address = unstructured
            .choose(&self.component_addresses[..])
            .unwrap()
            .clone();
        let non_fungible_resource_address = unstructured
            .choose(&self.non_fungible_resource_addresses[..])
            .unwrap()
            .clone();

        let mut global_addresses = {
            let package_address = unstructured.choose(&package_addresses[..]).unwrap().clone();
            vec![
                GlobalAddress::from(component_address),
                GlobalAddress::from(resource_address),
                GlobalAddress::from(package_address),
            ]
        };
        // TODO: if resource_address of not NonFungible resource is given then we got panic in get_mapped_substate
        // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: UnexpectedSize { expected: 2, actual: 1 }', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-stores/src/interface.rs:200:41
        let non_fungible_ids =
            self.get_non_fungible_local_id(component_address, non_fungible_resource_address);

        // To increase the chance of the successful transaction:
        // - fuzz fee amount for 5% of attempts
        // - use random component_address for 5% of attempts
        let fee = if unstructured.int_in_range(0..=100).unwrap() < 5 {
            Decimal::arbitrary(&mut unstructured).unwrap()
        } else {
            Decimal::from(100)
        };
        let fee_address = if unstructured.int_in_range(0..=100).unwrap() < 5 {
            component_address
        } else {
            FAUCET
        };

        builder.lock_fee(fee_address, fee);

        let mut i = 0;
        let instruction_cnt = unstructured.int_in_range(1..=INSTRUCTION_MAX_CNT).unwrap();

        while i < instruction_cnt && unstructured.len() > 0 {
            let next: u8 = unstructured
                .int_in_range(0..=ast::Instruction::COUNT as u8 - 1)
                .unwrap();

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
                    let bucket_id = *unstructured.choose(&buckets[..]).unwrap();

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
                7 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();

                    Some(Instruction::ClaimPackageRoyalty { package_address })
                }
                // ClearAuthZone
                8 => Some(Instruction::ClearAuthZone),
                // ClearSignatureProofs
                9 => Some(Instruction::ClearSignatureProofs),
                // CloneProof
                10 => {
                    let proof_id = *unstructured.choose(&proof_ids[..]).unwrap();

                    Some(Instruction::CloneProof { proof_id })
                }
                // CreateAccessController
                11 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let bucket_id = *unstructured.choose(&buckets[..]).unwrap();
                    let rule_set = RuleSet::arbitrary(&mut unstructured).unwrap();
                    let timed_recovery_delay_in_minutes =
                        <Option<u32>>::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
                        function_name: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
                        args: manifest_args!(bucket_id, rule_set, timed_recovery_delay_in_minutes),
                    })
                }
                // CreateAccount
                12 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                        function_name: ACCOUNT_CREATE_IDENT.to_string(),
                        args: to_manifest_value(
                            &AccountCreateInput::arbitrary(&mut unstructured).unwrap(),
                        ),
                    })
                }
                // CreateAccountAdvanced
                13 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let input = AccountCreateAdvancedInput::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                        function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateFungibleResource
                14 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let input =
                        FungibleResourceManagerCreateInput::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateFungibleResourceWithInitialSupply
                15 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let input = FungibleResourceManagerCreateWithInitialSupplyInput::arbitrary(
                        &mut unstructured,
                    )
                    .unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                            .to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateIdentity
                16 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let input = IdentityCreateInput::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: IDENTITY_BLUEPRINT.to_string(),
                        function_name: IDENTITY_CREATE_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateIdentityAdvanced
                17 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let input = IdentityCreateAdvancedInput::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: IDENTITY_BLUEPRINT.to_string(),
                        function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateNonFungibleResource
                18 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let input = NonFungibleResourceManagerCreateInput::arbitrary(&mut unstructured)
                        .unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }

                // CreateNonFungibleResourceWithInitialSupply
                19 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let input =
                        &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput::arbitrary(
                            &mut unstructured,
                        )
                        .unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name:
                            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                                .to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateProofFromAuthZone
                20 => Some(Instruction::CreateProofFromAuthZone { resource_address }),
                // CreateProofFromAuthZoneByAmount
                21 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::CreateProofFromAuthZoneByAmount {
                        amount,
                        resource_address,
                    })
                }
                // CreateProofFromAuthZoneByIds
                22 => Some(Instruction::CreateProofFromAuthZoneByIds {
                    ids: non_fungible_ids.clone(),
                    resource_address,
                }),
                // CreateProofFromBucket
                23 => {
                    let bucket_id = *unstructured.choose(&buckets[..]).unwrap();
                    Some(Instruction::CreateProofFromBucket { bucket_id })
                }
                // CreateValidator
                24 => {
                    let input = EpochManagerCreateValidatorInput { key: public_key };
                    Some(Instruction::CallMethod {
                        component_address,
                        method_name: EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // DropAllProofs
                25 => Some(Instruction::DropAllProofs),
                // DropProof
                26 => {
                    let proof_id = *unstructured.choose(&proof_ids[..]).unwrap();
                    Some(Instruction::DropProof { proof_id })
                }
                // MintFungible
                27 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::MintFungible {
                        resource_address,
                        amount,
                    })
                }
                // MintNonFungible
                28 => {
                    let input =
                        NonFungibleResourceManagerMintManifestInput::arbitrary(&mut unstructured)
                            .unwrap();

                    Some(Instruction::MintNonFungible {
                        resource_address,
                        args: to_manifest_value(&input),
                    })
                }
                // MintUuidNonFungible
                29 => {
                    let input = NonFungibleResourceManagerMintUuidManifestInput::arbitrary(
                        &mut unstructured,
                    )
                    .unwrap();

                    Some(Instruction::MintUuidNonFungible {
                        resource_address,
                        args: to_manifest_value(&input),
                    })
                }
                // PopFromAuthZone
                30 => Some(Instruction::PopFromAuthZone {}),
                // PublishPackage | PublishPackageAdvanced
                31 | 32 => {
                    // Publishing package involves a compilation by scrypto compiler.
                    // In case of AFL invoking external tool breaks fuzzing.
                    // For now we skip this step
                    // TODO: compile some packages before starting AFL and read compiled
                    //  binaries in AFL
                    None
                }
                // PushToAuthZone
                33 => {
                    let proof_id = *unstructured.choose(&proof_ids[..]).unwrap();
                    Some(Instruction::PushToAuthZone { proof_id })
                }
                // RecallResource
                34 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    let vault_id = {
                        let vaults = self
                            .runner
                            .get_component_vaults(component_address, resource_address);
                        if vaults.len() > 0 {
                            *unstructured.choose(&vaults[..]).unwrap()
                        } else {
                            InternalAddress::arbitrary(&mut unstructured).unwrap().into()
                        }
                    };
                    Some(Instruction::RecallResource {
                        vault_id: InternalAddress::new_or_panic(vault_id.into()),
                        amount,
                    })
                }
                // RemoveMetadata
                35 => {
                    global_addresses.push(
                        GlobalAddress::arbitrary(&mut unstructured).unwrap());
                    let entity_address = *unstructured.choose(&global_addresses[..]).unwrap();
                    let key = String::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::RemoveMetadata {
                        entity_address,
                        key,
                    })
                }
                // ReturnToWorktop
                36 => {
                    let bucket_id = *unstructured.choose(&buckets[..]).unwrap();
                    Some(Instruction::ReturnToWorktop { bucket_id })
                }
                // SetComponentRoyaltyConfig
                37 => {
                    let royalty_config = RoyaltyConfig::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::SetComponentRoyaltyConfig {
                        component_address,
                        royalty_config,
                    })
                }
                // SetMetadata
                38 => {
                    global_addresses.push(
                        GlobalAddress::arbitrary(&mut unstructured).unwrap());
                    let entity_address = *unstructured.choose(&global_addresses[..]).unwrap();
                    let key = String::arbitrary(&mut unstructured).unwrap();
                    let value = MetadataEntry::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::SetMetadata {
                        entity_address,
                        key,
                        value,
                    })
                }
                // SetMethodAccessRule
                39 => {
                    global_addresses.push(
                        GlobalAddress::arbitrary(&mut unstructured).unwrap());
                    let entity_address = *unstructured.choose(&global_addresses[..]).unwrap();
                    let key = MethodKey::arbitrary(&mut unstructured).unwrap();
                    let rule = AccessRule::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::SetMethodAccessRule {
                        entity_address,
                        key,
                        rule,
                    })
                }
                // SetPackageRoyaltyConfig
                40 => {
                    package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&package_addresses[..]).unwrap();
                    let royalty_config = BTreeMap::<String, RoyaltyConfig>::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::SetPackageRoyaltyConfig {
                        package_address,
                        royalty_config,
                    })
                }
                // TakeFromWorktop
                41 => Some(Instruction::TakeFromWorktop { resource_address }),
                // TakeFromWorktopByAmount
                42 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::TakeFromWorktopByAmount {
                        amount,
                        resource_address,
                    })
                }
                // TakeFromWorktopByIds
                43 => Some(Instruction::TakeFromWorktopByIds {
                    ids: non_fungible_ids.clone(),
                    resource_address,
                }),
                _ => unreachable!(
                    "Not all instructions (current count is {}) covered by this match, current instruction {}",
                    ast::Instruction::COUNT, next
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
        dbg!("manifest = {:?}", manifest);
        Ok(manifest)
    }

    pub fn fuzz_tx_manifest(&mut self, data: &[u8]) -> TxStatus {
        #[cfg(feature = "dummy_fuzzing")]
        let result = manifest_decode::<TransactionManifest>(data);
        #[cfg(not(feature = "dummy_fuzzing"))]
        let result = self.build_manifest(data);

        match result {
            #[allow(unused_mut)]
            Ok(mut manifest) => {
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

// This test tries is supposed to generate fuzz input data.
// It generates and executes manifest. If transaction successful then save the manifest data.
#[test]
#[cfg(not(feature = "dummy_fuzzing"))]
fn test_generate_fuzz_input_data() {
    use rand::{Rng, RngCore};
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(1234);
    let mut fuzzer = TxFuzzer::new();
    for _ in 0..5000 {
        let len = rng.gen_range(0..1024);
        let mut bytes: Vec<u8> = (0..len).map(|_| rng.gen_range(0..u8::MAX)).collect();
        rng.fill_bytes(&mut bytes[..]);

        let _result = catch_unwind(AssertUnwindSafe(|| {
            fuzzer.reset_runner();
            match fuzzer.fuzz_tx_manifest(&bytes[..]) {
                TxStatus::CommitSuccess => {
                    let m_hash = hash(&bytes);
                    let path = format!("manifest_{:?}.raw", m_hash);
                    std::fs::write(&path, bytes).unwrap();
                    println!("manifest dumped to file {}", &path);
                }
                _ => {}
            }
        }));
    }
}

// This test is supposed to generate fuzz input data.
// It runs radix-engine-tests tests with "dump_manifest_to_file" flag,
// which writes each used transaction manifest to file.
#[test]
#[cfg(feature = "dummy_fuzzing")]
fn test_generate_fuzz_input_data() {
    /*
    cargo nextest run -p radix-engine-tests --release --features dump_manifest_to_file
    mv ../radix-engine-tests/manifest_*.raw ${curr_path}/${raw_dir}
    */
    use std::fs;

    use std::io::{BufRead, BufReader};
    use std::process::Command;
    use std::process::Stdio;
    const WORK_DIR: &str = "/Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto";
    const PACKAGE: &str = "radix-engine-tests";

    let mut child = Command::new("cargo")
        .current_dir(WORK_DIR)
        .stdin(Stdio::null())
        .arg("nextest")
        .arg("run")
        .arg("-p")
        .arg(PACKAGE)
        .arg("--release")
        .arg("--features")
        .arg("dump_manifest_to_file")
        .spawn()
        .expect("failed to execute process");

    if let Some(stdout) = &mut child.stdout {
        let lines = BufReader::new(stdout).lines().enumerate().take(10);
        for (_, line) in lines {
            println!("{:?}", line);
        }
    }

    child.wait().expect("failed to wait");

    let entries = fs::read_dir(format!("{}/{}", WORK_DIR, PACKAGE)).unwrap();

    entries
        .filter_map(|entry| Some(entry.unwrap()))
        .for_each(|entry| {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_str().unwrap();
            if fname.ends_with(".raw") && fname.starts_with("manifest_") {
                fs::rename(entry.path(), fname).unwrap();
            }
        });
}

// Initialize static objects outside the fuzzing loop to assure deterministic instrumentation
// output across runs.
pub fn fuzz_tx_init_statics() {
    // Following code initializes secp256k1::SECP256K1 global static context
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(100).unwrap();
    let _public_key = private_key.public_key();
}
