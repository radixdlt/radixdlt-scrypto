use std::collections::BTreeMap;
use radix_common::math::Decimal;
use radix_common::prelude::{ComponentAddress, ManifestEncode, NonFungibleData, NonFungibleIdType, NonFungibleLocalId, ResourceAddress};
use radix_engine::vm::NativeVmExtension;
use radix_engine_interface::{burn_roles, deposit_roles, freeze_roles, metadata, mint_roles, non_fungible_data_update_roles, recall_roles, rule, withdraw_roles};
use radix_engine_interface::prelude::{FungibleResourceRoles, NonFungibleResourceRoles, OwnerRole, require};
use radix_transactions::builder::ManifestBuilder;
use crate::ledger_simulator::{EmptyNonFungibleData, LedgerSimulator, TestDatabase};
use crate::prelude::*;

pub trait LedgerSimulatorResourceExtension {
    fn create_fungible_resource_and_deposit(
        &mut self,
        owner_role: OwnerRole,
        resource_roles: FungibleResourceRoles,
        to: ComponentAddress,
    ) -> ResourceAddress;

    fn create_restricted_token(
        &mut self,
        account: ComponentAddress,
    ) -> (
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
    );

    fn create_everything_allowed_non_fungible_resource(&mut self, owner_role: OwnerRole) -> ResourceAddress;

    fn create_freezeable_token(&mut self, account: ComponentAddress) -> ResourceAddress;
    fn create_freezeable_non_fungible(&mut self, account: ComponentAddress) -> ResourceAddress;
    fn create_recallable_token(&mut self, account: ComponentAddress) -> ResourceAddress;
    fn create_restricted_burn_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress);
    fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress);
    fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress;
    fn create_non_fungible_resource_with_roles(
        &mut self,
        resource_roles: NonFungibleResourceRoles,
        account: ComponentAddress,
    ) -> ResourceAddress;
    fn create_non_fungible_resource_advanced(
        &mut self,
        resource_roles: NonFungibleResourceRoles,
        account: ComponentAddress,
        n: usize,
    ) -> ResourceAddress;
    fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress;
    fn create_mintable_burnable_fungible_resource(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress);
    fn create_freely_mintable_fungible_resource(
        &mut self,
        owner_role: OwnerRole,
        amount: Option<Decimal>,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress;
    fn create_freely_mintable_and_burnable_fungible_resource(
        &mut self,
        owner_role: OwnerRole,
        amount: Option<Decimal>,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress;
    fn create_freely_mintable_and_burnable_non_fungible_resource<T, V>(
        &mut self,
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        initial_supply: Option<T>,
        account: ComponentAddress,
    ) -> ResourceAddress
        where
            T: IntoIterator<Item = (NonFungibleLocalId, V)>,
            V: ManifestEncode + NonFungibleData;
}

impl<E: NativeVmExtension, D: TestDatabase> LedgerSimulatorResourceExtension for LedgerSimulator<E, D> {
    fn create_fungible_resource_and_deposit(
        &mut self,
        owner_role: OwnerRole,
        resource_roles: FungibleResourceRoles,
        to: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                owner_role,
                true,
                0,
                resource_roles,
                metadata!(),
                Some(5.into()),
            )
            .try_deposit_entire_worktop_or_abort(to, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    fn create_restricted_token(&mut self, account: ComponentAddress) -> (ResourceAddress, ResourceAddress, ResourceAddress, ResourceAddress, ResourceAddress, ResourceAddress, ResourceAddress, ResourceAddress) {
        let mint_auth = self.create_fungible_resource(dec!(1), 0, account);
        let burn_auth = self.create_fungible_resource(dec!(1), 0, account);
        let withdraw_auth = self.create_fungible_resource(dec!(1), 0, account);
        let recall_auth = self.create_fungible_resource(dec!(1), 0, account);
        let update_metadata_auth = self.create_fungible_resource(dec!(1), 0, account);
        let freeze_auth = self.create_fungible_resource(dec!(1), 0, account);
        let admin_auth = self.create_fungible_resource(dec!(1), 0, account);

        let token_address = self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => rule!(require(mint_auth));
                    minter_updater => rule!(require(admin_auth));
                },
                burn_roles: burn_roles! {
                    burner => rule!(require(burn_auth));
                    burner_updater => rule!(require(admin_auth));
                },
                freeze_roles: freeze_roles! {
                    freezer => rule!(require(freeze_auth));
                    freezer_updater => rule!(require(admin_auth));
                },
                recall_roles: recall_roles! {
                    recaller => rule!(require(recall_auth));
                    recaller_updater => rule!(require(admin_auth));
                },
                withdraw_roles: withdraw_roles! {
                    withdrawer => rule!(require(withdraw_auth));
                    withdrawer_updater => rule!(require(admin_auth));
                },
                deposit_roles: deposit_roles! {
                    depositor => rule!(allow_all);
                    depositor_updater => rule!(require(admin_auth));
                },
            },
            account,
        );

        (
            token_address,
            mint_auth,
            burn_auth,
            withdraw_auth,
            recall_auth,
            update_metadata_auth,
            freeze_auth,
            admin_auth,
        )
    }

    fn create_everything_allowed_non_fungible_resource(
        &mut self,
        owner_role: OwnerRole,
    ) -> ResourceAddress {
        let receipt = self.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_non_fungible_resource::<Vec<_>, ()>(
                    owner_role,
                    NonFungibleIdType::Integer,
                    false,
                    NonFungibleResourceRoles {
                        mint_roles: mint_roles! {
                            minter => rule!(allow_all);
                            minter_updater => rule!(allow_all);
                        },
                        burn_roles: burn_roles! {
                            burner => rule!(allow_all);
                            burner_updater => rule!(allow_all);
                        },
                        freeze_roles: freeze_roles! {
                            freezer => rule!(allow_all);
                            freezer_updater => rule!(allow_all);
                        },
                        recall_roles: recall_roles! {
                            recaller => rule!(allow_all);
                            recaller_updater => rule!(allow_all);
                        },
                        withdraw_roles: withdraw_roles! {
                            withdrawer => rule!(allow_all);
                            withdrawer_updater => rule!(allow_all);
                        },
                        deposit_roles: deposit_roles! {
                            depositor => rule!(allow_all);
                            depositor_updater => rule!(allow_all);
                        },
                        non_fungible_data_update_roles: non_fungible_data_update_roles! {
                            non_fungible_data_updater => rule!(allow_all);
                            non_fungible_data_updater_updater => rule!(allow_all);
                        },
                    },
                    metadata!(),
                    None,
                )
                .build(),
            vec![],
        );
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    fn create_freezeable_token(&mut self, account: ComponentAddress) -> ResourceAddress {
        self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                burn_roles: burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                },
                recall_roles: recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(deny_all);
                },
                freeze_roles: freeze_roles! {
                    freezer => rule!(allow_all);
                    freezer_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        )
    }

    fn create_freezeable_non_fungible(&mut self, account: ComponentAddress) -> ResourceAddress {
        self.create_non_fungible_resource_with_roles(
            NonFungibleResourceRoles {
                burn_roles: burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                },
                recall_roles: recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(deny_all);
                },
                freeze_roles: freeze_roles! {
                    freezer => rule!(allow_all);
                    freezer_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        )
    }

    fn create_recallable_token(&mut self, account: ComponentAddress) -> ResourceAddress {
        self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                recall_roles: recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        )
    }

    fn create_restricted_burn_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_fungible_resource(dec!(1), 0, account);

        let resource_address = self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                burn_roles: burn_roles! {
                    burner => rule!(require(auth_resource_address));
                    burner_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        );

        (auth_resource_address, resource_address)
    }

    fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let resource_address = self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                withdraw_roles: withdraw_roles! {
                    withdrawer => rule!(require(auth_resource_address));
                    withdrawer_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        );

        (auth_resource_address, resource_address)
    }

    fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress {
        self.create_non_fungible_resource_advanced(NonFungibleResourceRoles::default(), account, 3)
    }

    fn create_non_fungible_resource_with_roles(
        &mut self,
        resource_roles: NonFungibleResourceRoles,
        account: ComponentAddress,
    ) -> ResourceAddress {
        self.create_non_fungible_resource_advanced(resource_roles, account, 3)
    }

    fn create_non_fungible_resource_advanced(
        &mut self,
        resource_roles: NonFungibleResourceRoles,
        account: ComponentAddress,
        n: usize,
    ) -> ResourceAddress {
        let mut entries = BTreeMap::new();
        for i in 1..n + 1 {
            entries.insert(
                NonFungibleLocalId::integer(i as u64),
                EmptyNonFungibleData {},
            );
        }

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                resource_roles,
                metadata!(),
                Some(entries),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                divisibility,
                FungibleResourceRoles::default(),
                metadata!(),
                Some(amount),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    fn create_mintable_burnable_fungible_resource(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let admin_auth = self.create_non_fungible_resource(account);

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                1u8,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(require(admin_auth));
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(require(admin_auth));
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                None,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
        (admin_auth, resource_address)
    }

    fn create_freely_mintable_fungible_resource(
        &mut self,
        owner_role: OwnerRole,
        amount: Option<Decimal>,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                owner_role,
                true,
                divisibility,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                amount,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    fn create_freely_mintable_and_burnable_fungible_resource(
        &mut self,
        owner_role: OwnerRole,
        amount: Option<Decimal>,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                owner_role,
                true,
                divisibility,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                amount,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    fn create_freely_mintable_and_burnable_non_fungible_resource<T, V>(
        &mut self,
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        initial_supply: Option<T>,
        account: ComponentAddress,
    ) -> ResourceAddress
        where
            T: IntoIterator<Item = (NonFungibleLocalId, V)>,
            V: ManifestEncode + NonFungibleData,
    {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                owner_role,
                id_type,
                true,
                NonFungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                initial_supply,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }
}