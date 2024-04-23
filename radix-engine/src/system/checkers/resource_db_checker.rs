use crate::blueprints::consensus_manager::{
    ConsensusManagerCurrentProposalStatisticFieldPayload,
    ConsensusManagerCurrentValidatorSetFieldPayload, ConsensusManagerField,
};
use crate::blueprints::resource::{
    FungibleResourceManagerField, FungibleResourceManagerTotalSupplyFieldPayload,
    FungibleVaultBalanceFieldPayload, FungibleVaultField, NonFungibleResourceManagerField,
    NonFungibleResourceManagerTotalSupplyFieldPayload, NonFungibleVaultBalanceFieldPayload,
    NonFungibleVaultCollection, NonFungibleVaultField,
};
use crate::internal_prelude::*;
use crate::system::checkers::ApplicationChecker;
use radix_common::math::Decimal;
use radix_common::prelude::{scrypto_decode, RESOURCE_PACKAGE};
use radix_common::types::{NodeId, ResourceAddress};
use radix_engine_interface::api::FieldIndex;
use radix_engine_interface::blueprints::consensus_manager::CONSENSUS_MANAGER_BLUEPRINT;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, FUNGIBLE_VAULT_BLUEPRINT,
    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, NON_FUNGIBLE_VAULT_BLUEPRINT,
};
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;

#[derive(Debug, Default)]
pub struct ResourceCounter {
    expected: Option<Decimal>,
    tracking_supply: Decimal,
}

#[derive(Debug, Default)]
pub struct ResourceDatabaseChecker {
    resources: BTreeMap<ResourceAddress, ResourceCounter>,
    non_fungible_vaults: BTreeMap<NodeId, ResourceCounter>,
    fungible_vaults: BTreeMap<NodeId, Decimal>,
}

#[derive(Debug, Default)]
pub struct ResourceDatabaseCheckerResults {
    pub num_resources: usize,
    pub total_supply: BTreeMap<ResourceAddress, Decimal>,
    pub vaults: BTreeMap<NodeId, Decimal>,
}

impl ApplicationChecker for ResourceDatabaseChecker {
    type ApplicationCheckerResults = ResourceDatabaseCheckerResults;

    fn on_field(
        &mut self,
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        field_index: FieldIndex,
        value: &Vec<u8>,
    ) {
        if !info.blueprint_id.package_address.eq(&RESOURCE_PACKAGE) || module_id != ModuleId::Main {
            return;
        }

        match info.blueprint_id.blueprint_name.as_str() {
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT => {
                let field: FungibleResourceManagerField = field_index.try_into().unwrap();
                match field {
                    FungibleResourceManagerField::TotalSupply => {
                        let total_supply: FungibleResourceManagerTotalSupplyFieldPayload =
                            scrypto_decode(value).unwrap();
                        let address = ResourceAddress::new_or_panic(node_id.0);
                        let tracker = self.resources.entry(address).or_default();
                        tracker.expected =
                            Some(total_supply.fully_update_and_into_latest_version());
                    }
                    _ => {}
                }
            }
            FUNGIBLE_VAULT_BLUEPRINT => {
                let field: FungibleVaultField = field_index.try_into().unwrap();
                match field {
                    FungibleVaultField::Balance => {
                        let vault_balance: FungibleVaultBalanceFieldPayload =
                            scrypto_decode(value).unwrap();
                        let address = ResourceAddress::new_or_panic(
                            info.outer_obj_info.expect().into_node_id().0,
                        );
                        let amount = vault_balance
                            .fully_update_and_into_latest_version()
                            .amount();

                        if amount.is_negative() {
                            panic!("Found Fungible Vault negative balance");
                        }

                        let tracker = self.resources.entry(address).or_default();
                        tracker.tracking_supply =
                            tracker.tracking_supply.checked_add(amount).unwrap();

                        self.fungible_vaults.insert(node_id, amount);
                    }
                    _ => {}
                }
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT => {
                let field: NonFungibleResourceManagerField = field_index.try_into().unwrap();
                match field {
                    NonFungibleResourceManagerField::TotalSupply => {
                        let total_supply: NonFungibleResourceManagerTotalSupplyFieldPayload =
                            scrypto_decode(value).unwrap();
                        let address = ResourceAddress::new_or_panic(node_id.0);
                        let tracker = self.resources.entry(address).or_default();
                        tracker.expected =
                            Some(total_supply.fully_update_and_into_latest_version());
                    }
                    _ => {}
                }
            }
            NON_FUNGIBLE_VAULT_BLUEPRINT => {
                let field: NonFungibleVaultField = field_index.try_into().unwrap();
                match field {
                    NonFungibleVaultField::Balance => {
                        let vault_balance: NonFungibleVaultBalanceFieldPayload =
                            scrypto_decode(value).unwrap();
                        let address = ResourceAddress::new_or_panic(
                            info.outer_obj_info.expect().into_node_id().0,
                        );
                        let tracker = self.resources.entry(address).or_default();
                        let vault_balance = vault_balance.fully_update_and_into_latest_version();
                        tracker.tracking_supply = tracker
                            .tracking_supply
                            .checked_add(vault_balance.amount)
                            .unwrap();

                        let non_fungible_vault_tracker =
                            self.non_fungible_vaults.entry(node_id).or_default();

                        if vault_balance.amount.is_negative() {
                            panic!("Found Non Fungible Vault negative balance");
                        }

                        non_fungible_vault_tracker.expected = Some(vault_balance.amount);
                    }
                    _ => {}
                }
            }
            CONSENSUS_MANAGER_BLUEPRINT => {
                let field: ConsensusManagerField = field_index.try_into().unwrap();
                let mut validator_count = 0;
                let mut stats_count = 0;
                match field {
                    ConsensusManagerField::CurrentValidatorSet => {
                        let validator_set: ConsensusManagerCurrentValidatorSetFieldPayload =
                            scrypto_decode(value).unwrap();

                        let mut prev = Decimal::MAX;
                        for validator in validator_set
                            .fully_update_and_into_latest_version()
                            .validator_set
                            .validators_by_stake_desc
                        {
                            if validator.1.stake > prev {
                                panic!("Validator set are not in DESC order");
                            }
                            prev = validator.1.stake;
                            validator_count += 1;
                        }
                    }
                    ConsensusManagerField::CurrentProposalStatistic => {
                        let stats: ConsensusManagerCurrentProposalStatisticFieldPayload =
                            scrypto_decode(value).unwrap();
                        stats_count = stats
                            .fully_update_and_into_latest_version()
                            .validator_statistics
                            .len();
                    }
                    _ => {}
                }

                if validator_count != stats_count {
                    panic!("Validator count != proposal statistics count")
                }
            }
            _ => {}
        }
    }

    fn on_collection_entry(
        &mut self,
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        collection_index: CollectionIndex,
        _key: &Vec<u8>,
        _value: &Vec<u8>,
    ) {
        if !info.blueprint_id.package_address.eq(&RESOURCE_PACKAGE) || module_id != ModuleId::Main {
            return;
        }

        match info.blueprint_id.blueprint_name.as_str() {
            NON_FUNGIBLE_VAULT_BLUEPRINT => {
                let collection: NonFungibleVaultCollection = collection_index.try_into().unwrap();
                match collection {
                    NonFungibleVaultCollection::NonFungibleIndex => {
                        let non_fungible_vault_tracker =
                            self.non_fungible_vaults.entry(node_id).or_default();
                        non_fungible_vault_tracker.tracking_supply = non_fungible_vault_tracker
                            .tracking_supply
                            .checked_add(Decimal::one())
                            .unwrap();
                    }
                }
            }
            _ => {}
        }
    }

    fn on_finish(&self) -> Self::ApplicationCheckerResults {
        for (address, counter) in &self.non_fungible_vaults {
            if let Some(expected) = counter.expected {
                if !expected.eq(&counter.tracking_supply) {
                    panic!(
                        "Vault amount mismatch: {:?} index: {:?} tracked_supply: {:?}",
                        address, expected, counter.tracking_supply,
                    );
                }
            } else {
                panic!("Found non fungible vault with no amount index");
            }
        }

        let mut vaults: BTreeMap<NodeId, Decimal> = self
            .non_fungible_vaults
            .iter()
            .map(|(vault_id, counter)| (*vault_id, counter.tracking_supply))
            .collect();
        vaults.extend(self.fungible_vaults.clone());

        let mut total_supply = BTreeMap::new();

        for (address, tracker) in &self.resources {
            if let Some(total_supply) = tracker.expected {
                if !total_supply.eq(&tracker.tracking_supply) {
                    panic!(
                        "Total Supply mismatch: {:?} total_supply: {:?} tracked_supply: {:?}",
                        address, total_supply, tracker.tracking_supply,
                    );
                }
            }

            total_supply.insert(*address, tracker.tracking_supply);
        }

        ResourceDatabaseCheckerResults {
            num_resources: self.resources.len(),
            total_supply,
            vaults,
        }
    }
}
