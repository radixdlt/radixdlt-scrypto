use super::*;

pub fn get_builder_for_every_scenario() -> AllScenariosIterator {
    AllScenariosIterator::default()
}

#[derive(Default)]
pub struct AllScenariosIterator {
    index: usize,
}

impl Iterator for AllScenariosIterator {
    type Item = Box<dyn FnOnce(ScenarioCore) -> Box<dyn ScenarioInstance>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.index {
            1 => Some(Box::new(|core| {
                transfer_xrd::TransferXrdScenarioCreator::create(core)
            })),
            2 => Some(Box::new(|core| {
                radiswap::RadiswapScenarioCreator::create(core)
            })),
            3 => Some(Box::new(|core| metadata::MetadataScenario::create(core))),
            4 => Some(Box::new(|core| {
                fungible_resource::FungibleResourceScenarioCreator::create(core)
            })),
            5 => Some(Box::new(|core| {
                non_fungible_resource::NonFungibleResourceScenarioCreator::create(core)
            })),
            6 => {
                Some(Box::new(|core| {
                    account_authorized_depositors::AccountAuthorizedDepositorsScenarioCreator::create(core)
                }))
            }
            7 => Some(Box::new(|core| {
                global_n_owned::GlobalNOwnedScenarioCreator::create(core)
            })),
            8 => Some(Box::new(|core| {
                non_fungible_resource_with_remote_type::NonFungibleResourceWithRemoteTypeScenarioCreator::create(core)
            })),
            9 => Some(Box::new(|core| {
                kv_store_with_remote_type::KVStoreScenarioCreator::create(core)
            })),
            10 => Some(Box::new(|core| {
                max_transaction::MaxTransactionScenarioCreator::create(core)
            })),
            11 => Some(Box::new(|core| {
                royalties::RoyaltiesScenarioCreator::create(core)
            })),
            12 => Some(Box::new(|core| {
                maya_router::MayaRouterScenarioCreator::create(core)
            })),
            _ => None,
        }
    }
}
