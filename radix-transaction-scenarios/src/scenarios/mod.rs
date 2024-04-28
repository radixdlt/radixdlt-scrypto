use super::*;
use internal_prelude::*;

mod account_authorized_depositors;
mod account_locker;
mod fungible_resource;
mod global_n_owned;
mod kv_store_with_remote_type;
mod max_transaction;
mod maya_router;
mod metadata;
mod non_fungible_resource;
mod non_fungible_resource_with_remote_type;
mod radiswap;
mod royalties;
mod transfer_xrd;

// Add new scenarios TO THE BOTTOM OF THE LIST below.
lazy_static::lazy_static! {
    pub static ref ALL_SCENARIOS: IndexMap<String, Box<dyn ScenarioCreatorObjectSafe>> = {
        fn add<C: ScenarioCreatorObjectSafe>(map: &mut IndexMap<String, Box<dyn ScenarioCreatorObjectSafe>>, creator: C) {
            map.insert(
                creator.metadata().logical_name.to_string(),
                Box::new(creator),
            );
        }

        let mut map = Default::default();

        // Add new scenarios here TO THE BOTTOM OF THE LIST to register them
        // with the outside world.
        //
        // NOTE: ORDER MATTERS, as it affects the canonical order in which
        // scenarios get run, if multiple scenarios can get run at a given time.
        // This order therefore shouldn't be changed, to avoid affecting historic
        // execution on testnets.
        add(&mut map, transfer_xrd::TransferXrdScenarioCreator);
        add(&mut map, radiswap::RadiswapScenarioCreator);
        add(&mut map, metadata::MetadataScenarioCreator);
        add(&mut map, fungible_resource::FungibleResourceScenarioCreator);
        add(&mut map, non_fungible_resource::NonFungibleResourceScenarioCreator);
        add(&mut map, account_authorized_depositors::AccountAuthorizedDepositorsScenarioCreator);
        add(&mut map, global_n_owned::GlobalNOwnedScenarioCreator);
        add(&mut map, non_fungible_resource_with_remote_type::NonFungibleResourceWithRemoteTypeScenarioCreator);
        add(&mut map, kv_store_with_remote_type::KVStoreScenarioCreator);
        add(&mut map, max_transaction::MaxTransactionScenarioCreator);
        add(&mut map, royalties::RoyaltiesScenarioCreator);
        add(&mut map, account_locker::AccountLockerScenarioCreator);
        add(&mut map, maya_router::MayaRouterScenarioCreator);

        map
    };
}
