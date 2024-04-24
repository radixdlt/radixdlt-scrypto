use super::*;
use crate::internal_prelude::*;
use crate::object_modules::royalty::*;

/// Checks the state invariants of the component royalty module. Currently, the only invariant that
/// this checks is that the royalty amount is within the allowed limits. This does not check
/// anything about the accumulator vault as most of the things to check for are already handled by
/// the resource checker.
#[derive(Clone, Default, Debug)]
pub struct ComponentRoyaltyDatabaseChecker(Vec<LocatedError<ComponentRoyaltyDatabaseCheckerError>>);

impl ApplicationChecker for ComponentRoyaltyDatabaseChecker {
    type ApplicationCheckerResults = Vec<LocatedError<ComponentRoyaltyDatabaseCheckerError>>;

    fn on_collection_entry(
        &mut self,
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
        value: &Vec<u8>,
    ) {
        // Ignore if the module id is not the royalty module.
        if module_id != ModuleId::Royalty {
            return;
        }

        let location = ErrorLocation::CollectionEntry {
            info,
            node_id,
            module_id,
            collection_index,
            key: key.clone(),
            value: value.clone(),
        };

        let collection_index =
            ComponentRoyaltyCollection::from_repr(collection_index).expect("Impossible case!");
        match collection_index {
            ComponentRoyaltyCollection::MethodAmountKeyValue => {
                let _key = scrypto_decode::<ComponentRoyaltyMethodAmountKeyPayload>(&key)
                    .expect("Impossible Case.");
                let value = scrypto_decode::<ComponentRoyaltyMethodAmountEntryPayload>(&value)
                    .expect("Impossible Case.");

                self.check_royalty_amount(value, location);
            }
        }
    }

    fn on_finish(&self) -> Self::ApplicationCheckerResults {
        self.0.clone()
    }
}

impl ComponentRoyaltyDatabaseChecker {
    pub fn check_royalty_amount(
        &mut self,
        royalty_amount: ComponentRoyaltyMethodAmountEntryPayload,
        location: ErrorLocation,
    ) {
        let royalty_amount = royalty_amount.fully_update_and_into_latest_version();
        let max_royalty_in_xrd = Decimal::from_str(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();
        let max_royalty_in_usd = max_royalty_in_xrd / Decimal::from_str(USD_PRICE_IN_XRD).unwrap();

        match royalty_amount {
            RoyaltyAmount::Free => {}
            RoyaltyAmount::Xrd(amount) => {
                if amount.is_negative() {
                    self.0.push(LocatedError::new(
                        location,
                        ComponentRoyaltyDatabaseCheckerError::NegativeRoyaltyAmount(royalty_amount),
                    ))
                } else if amount > max_royalty_in_xrd {
                    self.0.push(LocatedError::new(
                        location,
                        ComponentRoyaltyDatabaseCheckerError::RoyaltyAmountExceedsMaximum {
                            amount: royalty_amount,
                            maximum: amount,
                        },
                    ))
                }
            }
            RoyaltyAmount::Usd(amount) => {
                if amount.is_negative() {
                    self.0.push(LocatedError::new(
                        location,
                        ComponentRoyaltyDatabaseCheckerError::NegativeRoyaltyAmount(royalty_amount),
                    ))
                } else if amount > max_royalty_in_usd {
                    self.0.push(LocatedError::new(
                        location,
                        ComponentRoyaltyDatabaseCheckerError::RoyaltyAmountExceedsMaximum {
                            amount: royalty_amount,
                            maximum: amount,
                        },
                    ))
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum ComponentRoyaltyDatabaseCheckerError {
    /// Negative royalty amounts are not permitted.
    NegativeRoyaltyAmount(RoyaltyAmount),
    /// Royalty amounts exceeding maximums are not permitted.
    RoyaltyAmountExceedsMaximum {
        amount: RoyaltyAmount,
        maximum: Decimal,
    },
}
