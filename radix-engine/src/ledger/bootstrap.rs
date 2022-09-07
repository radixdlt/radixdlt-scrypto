use crate::constants::DEFAULT_MAX_CALL_DEPTH;
use crate::engine::Track;
use crate::engine::{ExecutionTrace, Kernel, SystemApi, TrackReceipt};
use crate::fee::FeeReserve;
use crate::fee::UnlimitedLoanFeeReserve;
use crate::ledger::{ReadableSubstateStore, TypedInMemorySubstateStore, WriteableSubstateStore};
use crate::transaction::TransactionResult;
use crate::types::ResourceMethodAuthKey::Withdraw;
use crate::types::*;
use scrypto::prelude::Bucket;
use transaction::model::ExecutableInstruction;
use transaction::validation::{IdAllocator, IdSpace};

#[derive(TypeId, Encode, Decode)]
struct SystemComponentState {
    xrd: scrypto::resource::Vault,
}

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 24_000_000_000i128;

use crate::model::*;
use crate::wasm::{DefaultWasmEngine, InstructionCostRules, WasmInstrumenter, WasmMeteringParams};

pub struct GenesisReceipt {
    pub sys_faucet_package: PackageAddress,
    pub sys_utils_package: PackageAddress,
    pub account_package: PackageAddress,
    pub ecdsa_token: ResourceAddress,
    pub system_token: ResourceAddress,
    pub xrd_token: ResourceAddress,
    pub faucet_component: ComponentAddress,
}

// TODO: This would be much better handled if bootstrap was implemented as an executed transaction
// TODO: rather than a state snapshot.
pub fn execute_genesis<'s, R: FeeReserve>(
    mut track: Track<'s, R>,
) -> (TrackReceipt, GenesisReceipt) {
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let mut execution_trace = ExecutionTrace::new();

    let mut kernel = Kernel::new(
        Hash([0u8; Hash::LENGTH]),
        vec![],
        true,
        DEFAULT_MAX_CALL_DEPTH,
        &mut track,
        &mut wasm_engine,
        &mut wasm_instrumenter,
        WasmMeteringParams::new(InstructionCostRules::tiered(1, 5, 10, 5000), 512),
        &mut execution_trace,
        vec![],
    );

    let mut id_allocator = IdAllocator::new(IdSpace::Transaction);

    let create_sys_faucet_package = {
        let sys_faucet_package =
            extract_package(include_bytes!("../../../assets/sys_faucet.wasm").to_vec())
                .expect("Failed to construct sys-faucet package");
        ExecutableInstruction::PublishPackage {
            package: scrypto_encode(&sys_faucet_package),
        }
    };
    let create_sys_utils_package = {
        let sys_utils_package =
            extract_package(include_bytes!("../../../assets/sys_utils.wasm").to_vec())
                .expect("Failed to construct sys-utils package");
        ExecutableInstruction::PublishPackage {
            package: scrypto_encode(&sys_utils_package),
        }
    };
    let create_account_package = {
        let account_package =
            extract_package(include_bytes!("../../../assets/account.wasm").to_vec())
                .expect("Failed to construct account package");
        ExecutableInstruction::PublishPackage {
            package: scrypto_encode(&account_package),
        }
    };
    let create_ecdsa_token = {
        let metadata: HashMap<String, String> = HashMap::new();
        let mut ecdsa_resource_auth = HashMap::new();
        ecdsa_resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));
        let initial_supply: Option<MintParams> = None;

        // TODO: Remove nasty circular dependency on SYS_UTILS_PACKAGE
        ExecutableInstruction::CallFunction {
            package_address: SYS_UTILS_PACKAGE,
            blueprint_name: "SysUtils".to_string(),
            method_name: "new_resource".to_string(),
            args: args!(
                ResourceType::NonFungible,
                metadata,
                ecdsa_resource_auth,
                initial_supply
            ),
        }
    };

    // TODO: Perhaps combine with ecdsa token?
    let create_system_token = {
        let metadata: HashMap<String, String> = HashMap::new();
        let mut access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)> =
            HashMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));
        let initial_supply: Option<MintParams> = None;

        // TODO: Remove nasty circular dependency on SYS_UTILS_PACKAGE
        ExecutableInstruction::CallFunction {
            package_address: SYS_UTILS_PACKAGE,
            blueprint_name: "SysUtils".to_string(),
            method_name: "new_resource".to_string(),
            args: args!(
                ResourceType::NonFungible,
                metadata,
                access_rules,
                initial_supply
            ),
        }
    };

    let create_xrd_token = {
        let mut metadata = HashMap::new();
        metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
        metadata.insert("name".to_owned(), XRD_NAME.to_owned());
        metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
        metadata.insert("url".to_owned(), XRD_URL.to_owned());

        let mut access_rules = HashMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));

        let initial_supply: Option<MintParams> = Option::Some(MintParams::Fungible {
            amount: XRD_MAX_SUPPLY.into(),
        });

        ExecutableInstruction::CallFunction {
            package_address: SYS_UTILS_PACKAGE,
            blueprint_name: "SysUtils".to_string(),
            method_name: "new_resource".to_string(),
            args: args!(
                ResourceType::Fungible { divisibility: 18 },
                metadata,
                access_rules,
                initial_supply
            ),
        }
    };

    let take_xrd = ExecutableInstruction::TakeFromWorktop {
        resource_address: RADIX_TOKEN,
    };

    let create_xrd_faucet = {
        let bucket = Bucket(id_allocator.new_bucket_id().unwrap());
        ExecutableInstruction::CallFunction {
            package_address: SYS_FAUCET_PACKAGE,
            blueprint_name: "Faucet".to_string(),
            method_name: "new".to_string(),
            args: args!(bucket),
        }
    };

    let result = kernel
        .invoke_function(
            FnIdentifier::Native(NativeFnIdentifier::TransactionProcessor(
                TransactionProcessorFnIdentifier::Run,
            )),
            ScryptoValue::from_typed(&TransactionProcessorRunInput {
                instructions: vec![
                    create_sys_faucet_package,
                    create_sys_utils_package,
                    create_account_package,
                    create_ecdsa_token,
                    create_system_token,
                    create_xrd_token,
                    take_xrd,
                    create_xrd_faucet,
                ],
            }),
        )
        .unwrap();

    let invoke_result: Vec<Vec<u8>> = scrypto_decode(&result.raw).unwrap();
    let sys_faucet_package: PackageAddress = scrypto_decode(&invoke_result[0]).unwrap();
    let sys_utils_package: PackageAddress = scrypto_decode(&invoke_result[1]).unwrap();
    let account_package: PackageAddress = scrypto_decode(&invoke_result[2]).unwrap();
    let (ecdsa_token, _bucket): (ResourceAddress, Option<Bucket>) =
        scrypto_decode(&invoke_result[3]).unwrap();
    let (system_token, _bucket): (ResourceAddress, Option<Bucket>) =
        scrypto_decode(&invoke_result[4]).unwrap();
    let (xrd_token, _bucket): (ResourceAddress, Option<Bucket>) =
        scrypto_decode(&invoke_result[5]).unwrap();
    let faucet_component: ComponentAddress = scrypto_decode(&invoke_result[7]).unwrap();

    track.create_uuid_substate(SubstateId::System, System { epoch: 0 }, true);

    let resource_changes = execution_trace.to_receipt().resource_changes;

    let track_receipt = track.finalize(Ok(invoke_result), resource_changes);

    (
        track_receipt,
        GenesisReceipt {
            sys_faucet_package,
            sys_utils_package,
            account_package,
            ecdsa_token,
            system_token,
            xrd_token,
            faucet_component,
        },
    )
}

pub fn bootstrap<S>(substate_store: &mut S) -> GenesisReceipt
where
    S: ReadableSubstateStore + WriteableSubstateStore,
{
    if substate_store
        .get_substate(&SubstateId::ResourceManager(RADIX_TOKEN))
        .is_none()
    {
        let track = Track::new(substate_store, UnlimitedLoanFeeReserve::default());
        let (track_receipt, bootstrap_receipt) = execute_genesis(track);
        if let TransactionResult::Commit(c) = track_receipt.result {
            c.state_updates.commit(substate_store);
        } else {
            panic!("Failed to bootstrap")
        }
        bootstrap_receipt
    } else {
        let mut temporary_substate_store = TypedInMemorySubstateStore::new();
        let track = Track::new(
            &mut temporary_substate_store,
            UnlimitedLoanFeeReserve::default(),
        );
        let (_track_receipt, bootstrap_receipt) = execute_genesis(track);
        bootstrap_receipt
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::Track;
    use crate::fee::UnlimitedLoanFeeReserve;
    use crate::ledger::{execute_genesis, TypedInMemorySubstateStore};
    use scrypto::constants::ACCOUNT_PACKAGE;
    use scrypto::prelude::{
        ECDSA_TOKEN, RADIX_TOKEN, SYSTEM_TOKEN, SYS_FAUCET_COMPONENT, SYS_FAUCET_PACKAGE,
        SYS_UTILS_PACKAGE,
    };

    #[test]
    fn bootstrap_receipt_should_match_constants() {
        let mut temporary_substate_store = TypedInMemorySubstateStore::new();
        let track = Track::new(
            &mut temporary_substate_store,
            UnlimitedLoanFeeReserve::default(),
        );
        let (_track_receipt, bootstrap_receipt) = execute_genesis(track);

        assert_eq!(bootstrap_receipt.sys_faucet_package, SYS_FAUCET_PACKAGE);
        assert_eq!(bootstrap_receipt.sys_utils_package, SYS_UTILS_PACKAGE);
        assert_eq!(bootstrap_receipt.account_package, ACCOUNT_PACKAGE);
        assert_eq!(bootstrap_receipt.ecdsa_token, ECDSA_TOKEN);
        assert_eq!(bootstrap_receipt.system_token, SYSTEM_TOKEN);
        assert_eq!(bootstrap_receipt.xrd_token, RADIX_TOKEN);
        assert_eq!(bootstrap_receipt.faucet_component, SYS_FAUCET_COMPONENT);
    }
}
