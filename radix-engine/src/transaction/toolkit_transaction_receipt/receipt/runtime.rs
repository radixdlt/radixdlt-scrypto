use super::base::*;
use super::serializable::SerializableToolkitTransactionReceipt;
use crate::blueprints::resource::*;
use crate::object_modules::metadata::*;
use crate::system::system_modules::execution_trace::WorktopChange;
use crate::system::system_substates::*;
use crate::transaction::toolkit_transaction_receipt::error::*;
use crate::transaction::toolkit_transaction_receipt::ContextualTryInto;
use crate::transaction::*;
use radix_common::prelude::*;
use radix_engine_interface::prelude::{MetadataValue, *};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTypeSelector;
impl TypeSelector for RuntimeTypeSelector {
    type Usize = usize;
    type Bytes = Vec<u8>;
    type Decimal = Decimal;

    type NodeId = NodeId;
    type NonFungibleGlobalId = NonFungibleGlobalId;

    type MetadataValue = MetadataValue;

    type WorktopChange = WorktopChange;
}

/// The runtime toolkit transaction receipt.
pub type RuntimeToolkitTransactionReceipt = ToolkitTransactionReceipt<RuntimeTypeSelector>;

impl RuntimeToolkitTransactionReceipt {
    pub fn into_serializable_receipt(
        self,
        address_encoder: &AddressBech32Encoder,
    ) -> Result<SerializableToolkitTransactionReceipt, ToolkitReceiptError> {
        self.contextual_try_into(AddressBech32Encoder {
            hrp_set: address_encoder.hrp_set.clone(),
        })
    }
}

impl TryFrom<VersionedTransactionReceipt> for RuntimeToolkitTransactionReceipt {
    type Error = ToolkitReceiptError;

    fn try_from(value: VersionedTransactionReceipt) -> Result<Self, Self::Error> {
        match TransactionReceiptVersions::from(value) {
            TransactionReceiptVersions::V1(receipt) => receipt.try_into(),
        }
    }
}

impl TryFrom<TransactionReceiptV1> for RuntimeToolkitTransactionReceipt {
    type Error = ToolkitReceiptError;

    fn try_from(value: TransactionReceiptV1) -> Result<Self, Self::Error> {
        match value {
            TransactionReceiptV1 {
                result:
                    TransactionResult::Commit(CommitResult {
                        outcome: TransactionOutcome::Success(..),
                        state_update_summary,
                        state_updates,
                        execution_trace: Some(execution_trace),
                        application_events,
                        ..
                    }),
                fee_summary,
                ..
            } => Ok(Self::CommitSuccess {
                state_updates_summary: StateUpdatesSummary {
                    new_entities: state_update_summary
                        .new_components
                        .into_iter()
                        .map(|value| value.into_node_id())
                        .chain(
                            state_update_summary
                                .new_resources
                                .into_iter()
                                .map(|value| value.into_node_id()),
                        )
                        .chain(
                            state_update_summary
                                .new_packages
                                .into_iter()
                                .map(|value| value.into_node_id()),
                        )
                        .collect(),
                    // We get the metadata updates from the events.
                    metadata_updates: application_events
                        .iter()
                        .fold(
                            IndexMap::<NodeId, IndexMap<String, MetadataUpdate<RuntimeTypeSelector>>>::new(),
                            |mut acc, (EventTypeIdentifier(emitter, event_name), event_data)| {
                                // Check if this is a metadata emitter and if this is a metadata event.
                                match emitter {
                                    Emitter::Method(node_id, ModuleId::Metadata) => {
                                        match event_name.as_str() {
                                            SetMetadataEvent::EVENT_NAME => {
                                                let SetMetadataEvent { key, value } =
                                                    scrypto_decode::<SetMetadataEvent>(event_data)
                                                        .expect("Must succeed!");

                                                acc.entry(*node_id)
                                                    .or_default()
                                                    .insert(key, MetadataUpdate::Set(value));
                                                acc
                                            }
                                            RemoveMetadataEvent::EVENT_NAME => {
                                                let RemoveMetadataEvent { key } =
                                                    scrypto_decode::<RemoveMetadataEvent>(event_data)
                                                        .expect("Must succeed!");

                                                // If the metadata field was set and then unset in the same
                                                // tx then we remove the entry from the map.
                                                let map = acc.entry(*node_id).or_default();
                                                if map.swap_remove(&key).is_none() {
                                                    map.insert(key, MetadataUpdate::Delete);
                                                }

                                                acc
                                            }
                                            _ => acc,
                                        }
                                    }
                                    Emitter::Method(..) | Emitter::Function(..) => acc,
                                }
                            },
                        )
                        .into_iter()
                        .filter(|(_, metadata_updates)| !metadata_updates.is_empty())
                        .collect(),
                    // We get the non-fungible data updates from the state updates directly.
                    non_fungible_data_updates: state_updates
                        .by_node
                        .into_iter()
                        .filter_map(|(node_id, value)| {
                            ResourceAddress::try_from(node_id)
                                .map(|address| (address, value))
                                .ok()
                        })
                        .filter(|(resource_address, _)| !resource_address.is_fungible())
                        .filter_map(
                            |(
                                non_fungible_resource_address,
                                NodeStateUpdates::Delta {
                                    by_partition: updates_by_partition,
                                },
                            )| {
                                let partition_number = MAIN_BASE_PARTITION
                                    .at_offset(
                                        NonFungibleResourceManagerPartitionOffset::DataKeyValue
                                            .as_partition_offset(),
                                    )
                                    .unwrap();

                                let data_key_value_partition_updates =
                                    updates_by_partition.get(&partition_number)?;
                                let mut non_fungible_data = IndexMap::new();
                                match data_key_value_partition_updates {
                                    PartitionStateUpdates::Delta { by_substate } => {
                                        non_fungible_data.extend(by_substate.into_iter().filter_map(
                                            |(substate_key, database_update)| match (
                                                substate_key,
                                                database_update,
                                            ) {
                                                (SubstateKey::Map(key), DatabaseUpdate::Set(value)) => {
                                                    Some((key.clone(), value.clone()))
                                                }
                                                _ => None,
                                            },
                                        ))
                                    }
                                    PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                                        new_substate_values,
                                    }) => {
                                        non_fungible_data.extend(new_substate_values.into_iter().filter_map(
                                            |(key, value)| {
                                                if let SubstateKey::Map(key) = key {
                                                    Some((key.clone(), value.clone()))
                                                } else {
                                                    None
                                                }
                                            },
                                        ));
                                    }
                                }

                                non_fungible_data
                                    .into_iter()
                                    .map(|(key, value)| -> Option<_> {
                                        let key =
                                            scrypto_decode::<NonFungibleResourceManagerDataKeyPayload>(&key)
                                                .ok()
                                                .map(|id| {
                                                    NonFungibleGlobalId::new(
                                                        non_fungible_resource_address,
                                                        id.content,
                                                    )
                                                })?;
                                        let value = scrypto_decode::<
                                            KeyValueEntrySubstate<NonFungibleResourceManagerDataEntryPayload>,
                                        >(&value)
                                        .ok()
                                        .and_then(|value| value.into_value())
                                        .and_then(|value| scrypto_encode(&value).ok())?;
                                        Some((key, value))
                                    })
                                    .collect::<Option<IndexMap<_, _>>>()
                            },
                        )
                        .flatten()
                        .collect(),
                    // We get the newly minted non-fungibles from the events.
                    newly_minted_non_fungibles: application_events.iter().fold(
                        IndexSet::new(),
                        |mut acc, (EventTypeIdentifier(emitter, event_name), event_data)| match emitter {
                            Emitter::Method(node_id, ModuleId::Main) => {
                                match (ResourceAddress::try_from(*node_id), event_name.as_str()) {
                                    (Ok(resource_address), MintNonFungibleResourceEvent::EVENT_NAME) => {
                                        let MintNonFungibleResourceEvent { ids } =
                                            scrypto_decode(event_data).expect("Must succeed!");
                                        acc.extend(
                                            ids.into_iter().map(|value| {
                                                NonFungibleGlobalId::new(resource_address, value)
                                            }),
                                        );
                                        acc
                                    }
                                    (Ok(resource_address), BurnNonFungibleResourceEvent::EVENT_NAME) => {
                                        let BurnNonFungibleResourceEvent { ids } =
                                            scrypto_decode(event_data).expect("Must succeed!");
                                        ids.into_iter()
                                            .map(|value| NonFungibleGlobalId::new(resource_address, value))
                                            .for_each(|global_id| {
                                                acc.shift_remove(&global_id);
                                            });
                                        acc
                                    }
                                    _ => acc,
                                }
                            }
                            Emitter::Method(..) | Emitter::Function(..) => acc,
                        },
                    ),
                },
                worktop_changes: execution_trace
                    .worktop_changes(),
                fee_summary: FeeSummary {
                    execution_fees_in_xrd: fee_summary.total_execution_cost_in_xrd,
                    finalization_fees_in_xrd: fee_summary.total_finalization_cost_in_xrd,
                    storage_fees_in_xrd: fee_summary.total_storage_cost_in_xrd,
                    royalty_fees_in_xrd: fee_summary.total_royalty_cost_in_xrd,
                },
                locked_fees: LockedFees {
                    contingent: execution_trace.fee_locks.contingent_lock,
                    non_contingent: execution_trace.fee_locks.lock,
                },
            }),
            TransactionReceiptV1 {
                result:
                    TransactionResult::Commit(CommitResult {
                        outcome: TransactionOutcome::Success(..),
                        execution_trace: None,
                        ..
                    }),
                ..
            } => Err(ToolkitReceiptError::ReceiptLacksExecutionTrace),
            TransactionReceiptV1 {
                result:
                    TransactionResult::Commit(CommitResult {
                        outcome: TransactionOutcome::Failure(error),
                        ..
                    }),
                ..
            } => Ok(Self::CommitFailure {
                reason: format!("{error:?}"),
            }),
            TransactionReceiptV1 {
                result: TransactionResult::Reject(error),
                ..
            } => Ok(Self::Reject {
                reason: format!("{error:?}"),
            }),
            TransactionReceiptV1 {
                result: TransactionResult::Abort(error),
                ..
            } => Ok(Self::Abort {
                reason: format!("{error:?}"),
            }),
        }
    }
}
