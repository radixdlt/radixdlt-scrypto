use super::*;
use crate::system::bootstrap::*;

#[derive(Clone)]
pub struct BabylonSettings {
    pub genesis_data_chunks: Vec<GenesisDataChunk>,
    pub genesis_epoch: Epoch,
    pub consensus_manager_config: ConsensusManagerConfig,
    pub initial_time_ms: i64,
    pub initial_current_leader: Option<ValidatorIndex>,
    pub faucet_supply: Decimal,
}

impl BabylonSettings {
    pub fn test_default() -> Self {
        Self {
            genesis_data_chunks: vec![],
            genesis_epoch: Epoch::of(1),
            consensus_manager_config: ConsensusManagerConfig::test_default(),
            initial_time_ms: 1,
            initial_current_leader: Some(0),
            faucet_supply: *DEFAULT_TESTING_FAUCET_SUPPLY,
        }
    }

    pub fn test_complex() -> Self {
        let validator_key = Secp256k1PublicKey([0; 33]);
        let staker_address = ComponentAddress::preallocated_account_from_public_key(
            &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
        );
        let token_holder = ComponentAddress::preallocated_account_from_public_key(
            &PublicKey::Secp256k1(Secp256k1PrivateKey::from_u64(1).unwrap().public_key()),
        );
        let resource_address = ResourceAddress::new_or_panic(
            NodeId::new(
                EntityType::GlobalFungibleResourceManager as u8,
                &hash(vec![1, 2, 3]).lower_bytes(),
            )
            .0,
        );
        let stake = GenesisStakeAllocation {
            account_index: 0,
            xrd_amount: Decimal::one(),
        };
        let mut xrd_balances = Vec::new();
        let mut pub_key_accounts = Vec::new();

        for i in 0..20 {
            let pub_key = Secp256k1PrivateKey::from_u64((i + 1).try_into().unwrap())
                .unwrap()
                .public_key();
            let account_address = ComponentAddress::preallocated_account_from_public_key(&pub_key);
            pub_key_accounts.push((pub_key, account_address));
            xrd_balances.push((account_address, dec!("10")));
        }
        let genesis_resource = GenesisResource {
            reserved_resource_address: resource_address,
            metadata: vec![(
                "symbol".to_string(),
                MetadataValue::String("TST".to_string()),
            )],
            owner: None,
        };
        let resource_allocation = GenesisResourceAllocation {
            account_index: 0,
            amount: dec!("10"),
        };
        let genesis_data_chunks = vec![
            GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
            GenesisDataChunk::Stakes {
                accounts: vec![staker_address],
                allocations: vec![(validator_key, vec![stake])],
            },
            GenesisDataChunk::XrdBalances(xrd_balances),
            GenesisDataChunk::Resources(vec![genesis_resource]),
            GenesisDataChunk::ResourceBalances {
                accounts: vec![token_holder.clone()],
                allocations: vec![(resource_address.clone(), vec![resource_allocation])],
            },
        ];
        Self {
            genesis_data_chunks,
            genesis_epoch: Epoch::of(1),
            consensus_manager_config: ConsensusManagerConfig::test_default(),
            initial_time_ms: 1,
            initial_current_leader: Some(0),
            faucet_supply: Decimal::zero(),
        }
    }
}

impl UpdateSettings for BabylonSettings {
    type BatchGenerator = BabylonBatchGenerator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Babylon
    }

    fn all_enabled_as_default_for_network(_network: &NetworkDefinition) -> Self {
        Self::test_default()
    }

    fn all_disabled() -> Self {
        Self::test_default()
    }

    fn create_batch_generator(&self) -> Self::BatchGenerator {
        Self::BatchGenerator {
            settings: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct BabylonBatchGenerator {
    settings: BabylonSettings,
}

impl ProtocolUpdateBatchGenerator for BabylonBatchGenerator {
    fn generate_batch(
        &self,
        _store: &dyn SubstateDatabase,
        batch_group_index: usize,
        batch_index: usize,
    ) -> ProtocolUpdateBatch {
        match (batch_group_index, batch_index) {
            (0, 0) => ProtocolUpdateBatch::single(ProtocolUpdateTransactionDetails::flash(
                "flash",
                create_substate_flash_for_genesis().state_updates,
            )),
            (0, 1) => {
                ProtocolUpdateBatch::single(ProtocolUpdateTransactionDetails::genesis_transaction(
                    "bootstrap",
                    create_system_bootstrap_transaction(
                        self.settings.genesis_epoch,
                        self.settings.consensus_manager_config.clone(),
                        self.settings.initial_time_ms,
                        self.settings.initial_current_leader,
                        self.settings.faucet_supply,
                    ),
                ))
            }
            (1, batch_index) => {
                let chunk = self
                    .settings
                    .genesis_data_chunks
                    .get(batch_index)
                    .unwrap()
                    .clone();
                let chunk_number = batch_index;
                let transaction =
                    create_genesis_data_ingestion_transaction(&GENESIS_HELPER, chunk, chunk_number);
                ProtocolUpdateBatch::single(ProtocolUpdateTransactionDetails::genesis_transaction(
                    &format!("chunk-{chunk_number:04}"),
                    transaction,
                ))
            }
            (2, 0) => {
                ProtocolUpdateBatch::single(ProtocolUpdateTransactionDetails::genesis_transaction(
                    "wrap-up",
                    create_genesis_wrap_up_transaction(),
                ))
            }
            _ => {
                panic!("batch index out of range")
            }
        }
    }

    fn batch_count(&self, batch_group_index: usize) -> usize {
        match batch_group_index {
            0 => 2,
            1 => self.settings.genesis_data_chunks.len(),
            2 => 1,
            _ => panic!("Invalid batch_group_index: {batch_group_index}"),
        }
    }

    fn batch_group_descriptors(&self) -> Vec<String> {
        vec![
            "Bootstrap".to_string(),
            "Chunks".to_string(),
            "WrapUp".to_string(),
        ]
    }
}
