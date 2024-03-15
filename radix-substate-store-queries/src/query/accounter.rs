use super::{StateTreeTraverser, StateTreeVisitor};
use radix_common::prelude::*;
use radix_engine_interface::prelude::*;
use radix_substate_store_interface::interface::SubstateDatabase;

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
        state_tree_visitor.traverse_subtree(None, node_id)
    }

    pub fn close(self) -> Accounting {
        self.accounting
    }
}

pub struct Accounting {
    pub balances: HashMap<ResourceAddress, Decimal>,
    pub non_fungibles: HashMap<ResourceAddress, HashSet<NonFungibleLocalId>>,
}

impl Accounting {
    pub fn new() -> Self {
        Accounting {
            balances: hash_map_new(),
            non_fungibles: hash_map_new(),
        }
    }

    pub fn add_fungible_vault(
        &mut self,
        address: &ResourceAddress,
        resource: &LiquidFungibleResource,
    ) {
        let entry = self.balances.entry(*address).or_default();
        // NOTE: Decimal arithmetic operation safe unwrap.
        //       Resources have a mint limit below the Decimal max
        *entry = entry
            .checked_add(resource.amount())
            .expect("Resource overflow despite mint limit")
    }

    pub fn add_non_fungible_vault(
        &mut self,
        address: &ResourceAddress,
        resource: &LiquidNonFungibleVault,
    ) {
        let entry = self.balances.entry(*address).or_default();
        *entry = entry.checked_add(resource.amount).unwrap()
    }

    pub fn add_non_fungible(&mut self, address: &ResourceAddress, id: &NonFungibleLocalId) {
        self.non_fungibles
            .entry(*address)
            .or_default()
            .insert(id.clone());
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
        _address: &ResourceAddress,
        _resource: &LiquidNonFungibleVault,
    ) {
    }

    fn visit_non_fungible(
        &mut self,
        _vault_id: NodeId,
        address: &ResourceAddress,
        id: &NonFungibleLocalId,
    ) {
        self.add_non_fungible(address, id);
    }
}
