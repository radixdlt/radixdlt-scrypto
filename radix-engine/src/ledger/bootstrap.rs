use crate::ledger::{ReadableSubstateStore, SubstateIdGenerator, WriteableSubstateStore};
use sbor::*;
use scrypto::rule;
use scrypto::buffer::*;
use scrypto::constants::*;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::prelude::LOCKED;
use scrypto::resource::ResourceMethod::Withdraw;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::vec;

#[derive(TypeId, Encode, Decode)]
struct SystemComponentState {
    xrd: scrypto::resource::Vault,
}

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 24_000_000_000i128;
const XRD_VAULT_ID: VaultId = (Hash([0u8; 32]), 0);
const XRD_VAULT: scrypto::resource::Vault = scrypto::resource::Vault(XRD_VAULT_ID);

const SYSTEM_COMPONENT_NAME: &str = "System";

use crate::model::*;

pub fn bootstrap<S: ReadableSubstateStore + WriteableSubstateStore>(substate_store: &mut S) {
    let package: Option<Package> = substate_store
        .get_decoded_substate(&SYSTEM_PACKAGE)
        .map(|(package, _)| package);
    if package.is_none() {
        let nonce = substate_store.get_nonce();
        substate_store.increase_nonce();
        let tx_hash = hash(nonce.to_le_bytes());
        let mut id_gen = SubstateIdGenerator::new(tx_hash);

        // System package
        let system_package =
            Package::new(include_bytes!("../../../assets/system.wasm").to_vec()).unwrap();
        substate_store.put_encoded_substate(&SYSTEM_PACKAGE, &system_package, id_gen.next());

        // Account package
        let account_package =
            Package::new(include_bytes!("../../../assets/account.wasm").to_vec()).unwrap();
        substate_store.put_encoded_substate(&ACCOUNT_PACKAGE, &account_package, id_gen.next());

        // Radix token resource address
        let mut metadata = HashMap::new();
        metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
        metadata.insert("name".to_owned(), XRD_NAME.to_owned());
        metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
        metadata.insert("url".to_owned(), XRD_URL.to_owned());

        let mut resource_auth = HashMap::new();
        resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));

        let mut xrd = ResourceManager::new(
            ResourceType::Fungible { divisibility: 18 },
            metadata,
            resource_auth,
        )
            .unwrap();
        substate_store.put_encoded_substate(&RADIX_TOKEN, &xrd, id_gen.next());
        let minted_xrd = xrd
            .mint_fungible(XRD_MAX_SUPPLY.into(), RADIX_TOKEN.clone())
            .unwrap();

        let mut ecdsa_resource_auth = HashMap::new();
        ecdsa_resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));
        let ecdsa_token = ResourceManager::new(
            ResourceType::NonFungible,
            HashMap::new(),
            ecdsa_resource_auth,
        )
            .unwrap();
        substate_store.put_encoded_substate(&ECDSA_TOKEN, &ecdsa_token, id_gen.next());

        // Instantiate system component
        let system_vault = Vault::new(minted_xrd);
        substate_store.put_encoded_child_substate(
            &SYSTEM_COMPONENT,
            &XRD_VAULT_ID,
            &system_vault,
            id_gen.next(),
        );

        let system_component = Component::new(
            SYSTEM_PACKAGE,
            SYSTEM_COMPONENT_NAME.to_owned(),
            vec![],
            scrypto_encode(&SystemComponentState { xrd: XRD_VAULT }),
        );
        substate_store.put_encoded_substate(&SYSTEM_COMPONENT, &system_component, id_gen.next());
    }
}