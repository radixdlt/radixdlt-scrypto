use super::{StateTreeTraverser, StateTreeVisitor};
use crate::interface::SubstateDatabase;
use radix_engine_interface::{
    blueprints::resource::{LiquidFungibleResource, LiquidNonFungibleResource},
    data::scrypto::model::NonFungibleLocalId,
    math::Decimal,
    types::{NodeId, ResourceAddress},
};
use radix_engine_interface::blueprints::resource::LiquidNonFungibleVault;
use sbor::rust::ops::AddAssign;
use sbor::rust::prelude::*;

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

    pub fn traverse(&mut self, node_id: NodeId) {
        let mut state_tree_visitor =
            StateTreeTraverser::new(self.substate_db, &mut self.accounting, 100);
        state_tree_visitor.traverse_all_descendents(None, node_id)
    }

    pub fn close(self) -> Accounting {
        self.accounting
    }
}

pub struct Accounting {
    pub balances: HashMap<ResourceAddress, Decimal>,
}

impl Accounting {
    pub fn new() -> Self {
        Accounting {
            balances: HashMap::new(),
        }
    }

    pub fn add_fungible_vault(
        &mut self,
        address: &ResourceAddress,
        resource: &LiquidFungibleResource,
    ) {
        self.balances
            .entry(*address)
            .or_default()
            .add_assign(resource.amount())
    }

    pub fn add_non_fungible_vault(
        &mut self,
        address: &ResourceAddress,
        resource: &LiquidNonFungibleVault,
    ) {
        self.balances
            .entry(*address)
            .or_default()
            .add_assign(resource.amount)
    }
}

impl StateTreeVisitor for Accounting {
    fn visit_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        address: &ResourceAddress,
        resource: &LiquidFungibleResource,
    ) {
        self.add_fungible_vault(address, resource);
    }

    fn visit_non_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        address: &ResourceAddress,
        resource: &LiquidNonFungibleVault,
    ) {
        self.add_non_fungible_vault(address, resource);
    }
}
