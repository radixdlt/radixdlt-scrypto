use crate::constants::GENESIS_CREATION_CREDIT;
use crate::engine::ResourceChange;
use crate::engine::Track;
use crate::engine::TrackReceipt;
use crate::fee::FeeReserve;
use crate::fee::FeeTable;
use crate::fee::SystemLoanFeeReserve;
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::model::Package;
use crate::transaction::TransactionResult;
use crate::types::ResourceMethodAuthKey::Withdraw;
use crate::types::*;

#[derive(TypeId, Encode, Decode)]
struct SystemComponentState {
    vault: scrypto::resource::Vault,
    transactions: scrypto::component::KeyValueStore<Hash, u64>,
}

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 1_000_000_000_000i128;
const XRD_VAULT_ID: VaultId = (Hash([0u8; 32]), 0);

const SYS_FAUCET_COMPONENT_NAME: &str = "SysFaucet";
const SYS_FAUCET_KEY_VALUE_STORE_ID: KeyValueStoreId = (Hash([0u8; 32]), 1);

use crate::model::*;

// TODO: This would be much better handled if bootstrap was implemented as an executed transaction
// TODO: rather than a state snapshot.
pub fn execute_genesis<'s, R: FeeReserve>(mut track: Track<'s, R>) -> TrackReceipt {
    let sys_faucet_code = include_bytes!("../../../assets/sys_faucet.wasm").to_vec();
    let sys_faucet_abi = scrypto_decode(include_bytes!("../../../assets/sys_faucet.abi"))
        .expect("Failed to construct sys-faucet package");
    track.create_uuid_substate(
        SubstateId::Package(SYS_FAUCET_PACKAGE),
        Package::new(sys_faucet_code, sys_faucet_abi).expect("Invalid sys-faucet package"),
        true,
    );
    let sys_utils_code = include_bytes!("../../../assets/sys_utils.wasm").to_vec();
    let sys_utils_abi = scrypto_decode(include_bytes!("../../../assets/sys_utils.abi"))
        .expect("Failed to construct sys-utils package");
    track.create_uuid_substate(
        SubstateId::Package(SYS_UTILS_PACKAGE),
        Package::new(sys_utils_code, sys_utils_abi).expect("Invalid sys-utils package"),
        true,
    );
    let account_code = include_bytes!("../../../assets/account.wasm").to_vec();
    let account_abi = scrypto_decode(include_bytes!("../../../assets/account.abi"))
        .expect("Failed to construct account package");
    track.create_uuid_substate(
        SubstateId::Package(ACCOUNT_PACKAGE),
        Package::new(account_code, account_abi).expect("Invalid account package"),
        true,
    );

    // Radix token resource address
    let mut metadata = HashMap::new();
    metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
    metadata.insert("name".to_owned(), XRD_NAME.to_owned());
    metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
    metadata.insert("url".to_owned(), XRD_URL.to_owned());

    let mut resource_auth = HashMap::new();
    resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));

    let mut xrd_resource_manager = ResourceManager::new(
        ResourceType::Fungible { divisibility: 18 },
        metadata,
        resource_auth,
    )
    .expect("Failed to construct XRD resource manager");
    let minted_xrd = xrd_resource_manager
        .mint_fungible(XRD_MAX_SUPPLY.into(), RADIX_TOKEN.clone())
        .expect("Failed to mint XRD");
    track.create_uuid_substate(
        SubstateId::ResourceManager(RADIX_TOKEN),
        xrd_resource_manager,
        true,
    );

    let mut ecdsa_secp256k1_resource_auth = HashMap::new();
    ecdsa_secp256k1_resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));
    let ecdsa_secp256k1_token = ResourceManager::new(
        ResourceType::NonFungible,
        HashMap::new(),
        ecdsa_secp256k1_resource_auth,
    )
    .expect("Failed to construct ECDSA resource manager");
    track.create_uuid_substate(
        SubstateId::ResourceManager(ECDSA_TOKEN),
        ecdsa_secp256k1_token,
        true,
    );

    let system_token =
        ResourceManager::new(ResourceType::NonFungible, HashMap::new(), HashMap::new())
            .expect("Failed to construct SYSTEM_TOKEN resource manager");
    track.create_uuid_substate(
        SubstateId::ResourceManager(SYSTEM_TOKEN),
        system_token,
        true,
    );

    let initial_xrd = ResourceChange {
        resource_address: RADIX_TOKEN,
        component_address: SYS_FAUCET_COMPONENT,
        vault_id: XRD_VAULT_ID,
        amount: minted_xrd.amount(),
    };

    let system_vault = VaultSubstate(minted_xrd);
    track.create_uuid_substate(SubstateId::Vault(XRD_VAULT_ID), system_vault, false);

    let sys_faucet_component_info = ComponentInfoSubstate::new(
        SYS_FAUCET_PACKAGE,
        SYS_FAUCET_COMPONENT_NAME.to_owned(),
        vec![],
    );
    let sys_faucet_component_state =
        ComponentStateSubstate::new(scrypto_encode(&SystemComponentState {
            vault: scrypto::resource::Vault(XRD_VAULT_ID),
            transactions: scrypto::component::KeyValueStore {
                id: SYS_FAUCET_KEY_VALUE_STORE_ID,
                key: PhantomData,
                value: PhantomData,
            },
        }));
    track.create_uuid_substate(
        SubstateId::ComponentInfo(SYS_FAUCET_COMPONENT),
        sys_faucet_component_info,
        true,
    );
    track.create_uuid_substate(
        SubstateId::ComponentState(SYS_FAUCET_COMPONENT),
        sys_faucet_component_state,
        true,
    );

    track.create_uuid_substate(SubstateId::System, System { epoch: 0 }, true);

    track.finalize(Ok(Vec::new()), vec![initial_xrd])
}

pub fn bootstrap<S>(mut substate_store: S) -> S
where
    S: ReadableSubstateStore + WriteableSubstateStore,
{
    if substate_store
        .get_substate(&SubstateId::Package(SYS_FAUCET_PACKAGE))
        .is_none()
    {
        let mut fee_reserve = SystemLoanFeeReserve::default();
        fee_reserve.credit(GENESIS_CREATION_CREDIT);
        let track = Track::new(&substate_store, fee_reserve, FeeTable::new());
        let receipt = execute_genesis(track);
        if let TransactionResult::Commit(c) = receipt.result {
            c.state_updates.commit(&mut substate_store);
        } else {
            panic!("Failed to bootstrap")
        }
    }
    substate_store
}
