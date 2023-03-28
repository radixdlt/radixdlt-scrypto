use super::{StateTreeTraverser, StateTreeVisitor, VaultInfoSubstate};
use crate::interface::SubstateDatabase;
use radix_engine_interface::{
    blueprints::resource::{LiquidFungibleResource, LiquidNonFungibleResource},
    math::Decimal,
    types::{NodeId, ResourceAddress},
};
use sbor::rust::prelude::{hash_map::Entry, *};

pub struct ResourceAccounter<'s, S: SubstateDatabase> {
    substate_db: &'s S,
    accounting: Accounting,
}

impl<'s, S: SubstateDatabase> ResourceAccounter<'s, S> {
    pub fn new(substate_db: &'s S) -> Self {
        ResourceAccounter {
            substate_db,
            accounting: Accounting::new(),
        }
    }

    pub fn add_resources(&mut self, node_id: NodeId) {
        let mut state_tree_visitor =
            StateTreeTraverser::new(self.substate_db, &mut self.accounting, 100);
        state_tree_visitor.traverse_all_descendents(None, node_id)
    }

    pub fn into_map(self) -> HashMap<ResourceAddress, Decimal> {
        self.accounting.balances
    }
}

struct Accounting {
    balances: HashMap<ResourceAddress, Decimal>,
}

impl Accounting {
    pub fn new() -> Self {
        Accounting {
            balances: HashMap::new(),
        }
    }

    pub fn add_fungible_vault(
        &mut self,
        info: &VaultInfoSubstate,
        resource: &LiquidFungibleResource,
    ) {
        match self.balances.entry(info.resource_address) {
            Entry::Occupied(mut e) => {
                let new_amount = resource.amount() + *e.get();
                e.insert(new_amount);
            }
            Entry::Vacant(e) => {
                e.insert(resource.amount());
            }
        }
    }

    pub fn add_non_fungible_vault(
        &mut self,
        info: &VaultInfoSubstate,
        resource: &LiquidNonFungibleResource,
    ) {
        match self.balances.entry(info.resource_address) {
            Entry::Occupied(mut e) => {
                let new_amount = resource.amount() + *e.get();
                e.insert(new_amount);
            }
            Entry::Vacant(e) => {
                e.insert(resource.amount());
            }
        }
    }
}

impl StateTreeVisitor for Accounting {
    fn visit_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        info: &VaultInfoSubstate,
        resource: &LiquidFungibleResource,
    ) {
        self.add_fungible_vault(info, resource);
    }

    fn visit_non_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        info: &VaultInfoSubstate,
        resource: &LiquidNonFungibleResource,
    ) {
        self.add_non_fungible_vault(info, resource);
    }
}
