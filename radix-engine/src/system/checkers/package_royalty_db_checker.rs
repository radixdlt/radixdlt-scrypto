use super::*;
use crate::blueprints::package::*;
use crate::internal_prelude::*;

/// Checks the following invariants of the substates of the package royalties:
/// 1. That the royalties are within their allowable limits.
/// 2. No package royalties may be defined for methods that do not exist in the package schema.
///
/// This does not check anything about the accumulator vault as most of the things to check for are
/// already handled by the resource checker.
#[derive(Clone, Debug)]
pub struct PackageRoyaltyDatabaseChecker<F>
where
    F: Fn(&BlueprintId, &str) -> bool,
{
    /// A callback function used to check if a function or method exist in the package definition or
    /// not. This callback will be called for each function seen in the package royalties to verify
    /// their existence in the package definition.
    function_existence_callback: F,

    /// The errors collected as the checker goes through the database.   
    errors: Vec<LocatedError<PackageRoyaltyDatabaseCheckerError>>,
}

impl<F> ApplicationChecker for PackageRoyaltyDatabaseChecker<F>
where
    F: Fn(&BlueprintId, &str) -> bool,
{
    type ApplicationCheckerResults = Vec<LocatedError<PackageRoyaltyDatabaseCheckerError>>;

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
            PackageCollection::from_repr(collection_index).expect("Impossible case!");
        match collection_index {
            PackageCollection::BlueprintVersionRoyaltyConfigKeyValue => {
                let _key = scrypto_decode::<PackageBlueprintVersionRoyaltyConfigKeyPayload>(&key)
                    .expect("Impossible Case.");
                let value =
                    scrypto_decode::<PackageBlueprintVersionRoyaltyConfigEntryPayload>(&value)
                        .expect("Impossible Case.");
                self.check_package_royalty_config(value, location);
            }
            /* Nothing else to check in the package substates. */
            _ => return,
        }
    }

    fn on_finish(&self) -> Self::ApplicationCheckerResults {
        self.errors.clone()
    }
}

impl PackageRoyaltyDatabaseChecker<fn(&BlueprintId, &str) -> bool> {
    pub fn new_without_function_existence_check() -> Self {
        /* All functions exist regardless of the input */
        Self::new(|_, _| true)
    }
}

impl<F> PackageRoyaltyDatabaseChecker<F>
where
    F: Fn(&BlueprintId, &str) -> bool,
{
    pub fn new(function_existence_callback: F) -> Self {
        Self {
            function_existence_callback,
            errors: Default::default(),
        }
    }

    pub fn check_package_royalty_config(
        &mut self,
        config: PackageBlueprintVersionRoyaltyConfigEntryPayload,
        location: ErrorLocation,
    ) {
        let royalty_config = config.fully_update_and_into_latest_version();

        match royalty_config {
            PackageRoyaltyConfig::Disabled => {}
            PackageRoyaltyConfig::Enabled(royalty_config) => {
                royalty_config.values().for_each(|royalty_amount| {
                    self.check_royalty_amount(*royalty_amount, location.clone())
                })
            }
        }
    }

    pub fn check_for_function_existence(
        &mut self,
        blueprint_id: &BlueprintId,
        function_name: &str,
        location: ErrorLocation,
    ) {
        let func = &self.function_existence_callback;
        if !func(blueprint_id, function_name) {
            self.errors.push(LocatedError::new(
                location,
                PackageRoyaltyDatabaseCheckerError::FunctionDoesNotExistForPackage(
                    function_name.to_owned(),
                ),
            ))
        }
    }

    pub fn check_royalty_amount(&mut self, royalty_amount: RoyaltyAmount, location: ErrorLocation) {
        let max_royalty_in_xrd = Decimal::from_str(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();
        let max_royalty_in_usd = max_royalty_in_xrd / Decimal::from_str(USD_PRICE_IN_XRD).unwrap();

        match royalty_amount {
            RoyaltyAmount::Free => {}
            RoyaltyAmount::Xrd(amount) => {
                if amount.is_negative() {
                    self.errors.push(LocatedError::new(
                        location,
                        PackageRoyaltyDatabaseCheckerError::NegativeRoyaltyAmount(royalty_amount),
                    ))
                } else if amount > max_royalty_in_xrd {
                    self.errors.push(LocatedError::new(
                        location,
                        PackageRoyaltyDatabaseCheckerError::RoyaltyAmountExceedsMaximum {
                            amount: royalty_amount,
                            maximum: amount,
                        },
                    ))
                }
            }
            RoyaltyAmount::Usd(amount) => {
                if amount.is_negative() {
                    self.errors.push(LocatedError::new(
                        location,
                        PackageRoyaltyDatabaseCheckerError::NegativeRoyaltyAmount(royalty_amount),
                    ))
                } else if amount > max_royalty_in_usd {
                    self.errors.push(LocatedError::new(
                        location,
                        PackageRoyaltyDatabaseCheckerError::RoyaltyAmountExceedsMaximum {
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
pub enum PackageRoyaltyDatabaseCheckerError {
    /// Negative royalty amounts are not permitted.
    NegativeRoyaltyAmount(RoyaltyAmount),
    /// Royalty amounts exceeding maximums are not permitted.
    RoyaltyAmountExceedsMaximum {
        amount: RoyaltyAmount,
        maximum: Decimal,
    },
    /// Encountered royalties defined for a function or method that does not exist in the package
    /// schema.
    FunctionDoesNotExistForPackage(String),
}
