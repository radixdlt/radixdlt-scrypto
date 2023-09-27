use super::*;
use crate::blueprints::util::*;
use crate::system::attached_modules::role_assignment::*;

use radix_engine_interface::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct RoleAssignmentDatabaseChecker {
    /// An optional vector of the initial role keys that the role-assignment module had been created
    /// with. An invariant that we have in the role-assignment module is that new role assignments
    /// can't be added after the creation of the role-assignment module. This check is done by the
    /// auth layer. If this is [`Some`], then the checker ensures that no role-keys are encountered
    /// of roles outside of the initial role-keys. If [`None`] then this check does not happen.
    initial_roles_keys: Option<Vec<ModuleRoleKey>>,

    /// The id of the node to check the role-assignment module of. If provided then all fields and
    /// collection entires belonging to other nodes are ignored.
    node_id: Option<NodeId>,

    /// A vector of all of the errors encountered when going through the RoleAssignment substates.
    errors: Vec<LocatedRoleAssignmentDatabaseCheckerError>,
}

impl ApplicationChecker for RoleAssignmentDatabaseChecker {
    type ApplicationCheckerResults = Vec<LocatedRoleAssignmentDatabaseCheckerError>;

    fn on_field(
        &mut self,
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        field_index: FieldIndex,
        value: &Vec<u8>,
    ) {
        // The method responsible for the addition of errors to the checker state. This method will
        // add the location information.
        let mut add_error = |error| {
            self.errors.push(LocatedRoleAssignmentDatabaseCheckerError {
                location: ErrorLocation::Field {
                    info: info.clone(),
                    node_id,
                    module_id,
                    field_index,
                    value: value.clone(),
                },
                error,
            })
        };

        // Ignore all fields that do not belong to the role-assignment module.
        if module_id != ModuleId::RoleAssignment {
            return;
        }
        match self.node_id {
            Some(state_node_id) if state_node_id != node_id => return,
            _ => {}
        }

        let Some(typed_field_index) = RoleAssignmentField::from_repr(field_index) else {
            add_error(RoleAssignmentDatabaseCheckerError::InvalidFieldIndex(
                field_index,
            ));
            return;
        };

        match typed_field_index {
            RoleAssignmentField::Owner => {
                let Ok(owner_role) = scrypto_decode::<RoleAssignmentOwnerFieldPayload>(&value)
                    .map(|value| value.into_latest())
                else {
                    add_error(
                        RoleAssignmentDatabaseCheckerError::FailedToDecodeFieldValue(
                            typed_field_index,
                            value.clone(),
                        ),
                    );
                    return;
                };
                Self::check_access_rule(owner_role.owner_role_entry.rule, &mut add_error)
            }
        };
    }

    fn on_collection_entry(
        &mut self,
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
        value: &Vec<u8>,
    ) {
        // The method responsible for the addition of errors to the checker state. This method will
        // add the location information.
        let mut add_error = |error| {
            self.errors.push(LocatedRoleAssignmentDatabaseCheckerError {
                location: ErrorLocation::CollectionEntry {
                    info: info.clone(),
                    node_id,
                    module_id,
                    collection_index,
                    key: key.clone(),
                    value: value.clone(),
                },
                error,
            })
        };

        // Ignore all collection entires that do not belong to the role-assignment module.
        if module_id != ModuleId::RoleAssignment {
            return;
        }
        match self.node_id {
            Some(state_node_id) if state_node_id != node_id => return,
            _ => {}
        }

        let Some(typed_collection_index) = RoleAssignmentCollection::from_repr(collection_index) else {
            add_error(RoleAssignmentDatabaseCheckerError::InvalidCollectionIndex(
                collection_index,
            ));
            return;
        };

        match typed_collection_index {
            RoleAssignmentCollection::AccessRuleKeyValue => {
                let Ok(module_role_key) =
                    scrypto_decode::<RoleAssignmentAccessRuleKeyContent>(&key)
                else {
                    add_error(
                        RoleAssignmentDatabaseCheckerError::FailedToDecodeCollectionKeyOrValue(
                            typed_collection_index,
                            key.clone(),
                        ),
                    );
                    return;
                };
                let Ok(access_rule) =
                    scrypto_decode::<RoleAssignmentAccessRuleEntryPayload>(&value)
                        .map(|value| value.into_latest())
                else {
                    add_error(
                        RoleAssignmentDatabaseCheckerError::FailedToDecodeCollectionKeyOrValue(
                            typed_collection_index,
                            value.clone(),
                        ),
                    );
                    return;
                };

                Self::check_access_rule(access_rule, &mut add_error);
                Self::check_is_role_key_reserved(&module_role_key, &mut add_error);
                Self::check_is_reserved_space(&module_role_key, &mut add_error);
                Self::check_role_key_length(&module_role_key, &mut add_error);
                Self::check_role_key_name(&module_role_key, &mut add_error);
                Self::check_against_initial_role_keys(
                    &module_role_key,
                    &self.initial_roles_keys,
                    &mut add_error,
                );
            }
        }
    }

    fn on_finish(&self) -> Self::ApplicationCheckerResults {
        self.errors.clone()
    }
}

impl RoleAssignmentDatabaseChecker {
    pub fn new(initial_roles_keys: Vec<ModuleRoleKey>, node_id: NodeId) -> Self {
        Self {
            initial_roles_keys: Some(initial_roles_keys),
            node_id: Some(node_id),
            errors: Default::default(),
        }
    }

    fn check_access_rule<F>(access_rule: AccessRule, add_error: &mut F)
    where
        F: FnMut(RoleAssignmentDatabaseCheckerError),
    {
        if let Err(error) = RoleAssignmentNativePackage::verify_access_rule(&access_rule) {
            add_error(RoleAssignmentDatabaseCheckerError::InvalidAccessRule(
                access_rule,
                error,
            ))
        }
    }

    fn check_is_role_key_reserved<F>(role_key: &ModuleRoleKey, add_error: &mut F)
    where
        F: FnMut(RoleAssignmentDatabaseCheckerError),
    {
        if RoleAssignmentNativePackage::is_reserved_role_key(&role_key.key) {
            add_error(RoleAssignmentDatabaseCheckerError::ReservedRoleKey(
                role_key.clone(),
            ))
        }
    }

    fn check_is_reserved_space<F>(role_key: &ModuleRoleKey, add_error: &mut F)
    where
        F: FnMut(RoleAssignmentDatabaseCheckerError),
    {
        if role_key.module == ModuleId::RoleAssignment {
            add_error(RoleAssignmentDatabaseCheckerError::RoleKeyInReservedSpace(
                role_key.clone(),
            ))
        }
    }

    fn check_role_key_length<F>(role_key: &ModuleRoleKey, add_error: &mut F)
    where
        F: FnMut(RoleAssignmentDatabaseCheckerError),
    {
        if role_key.key.key.len() > MAX_ROLE_NAME_LEN {
            add_error(
                RoleAssignmentDatabaseCheckerError::RoleKeyExceedsMaximumLength {
                    actual: role_key.key.key.len(),
                    module_role_key: role_key.clone(),
                    maximum_length: MAX_ROLE_NAME_LEN,
                },
            )
        }
    }

    fn check_role_key_name<F>(role_key: &ModuleRoleKey, add_error: &mut F)
    where
        F: FnMut(RoleAssignmentDatabaseCheckerError),
    {
        if let Err(error) = check_name(&role_key.key.key) {
            add_error(RoleAssignmentDatabaseCheckerError::RoleKeyNameCheckFailed(
                role_key.clone(),
                error,
            ))
        }
    }

    fn check_against_initial_role_keys<F>(
        role_key: &ModuleRoleKey,
        initial_role_keys: &Option<Vec<ModuleRoleKey>>,
        add_error: &mut F,
    ) where
        F: FnMut(RoleAssignmentDatabaseCheckerError),
    {
        if let Some(ref initial_role_keys) = initial_role_keys {
            if !initial_role_keys.contains(&role_key) {
                add_error(
                    RoleAssignmentDatabaseCheckerError::RoleKeyWasCreatedAfterInitialization {
                        initial_role_keys: initial_role_keys.clone(),
                        role_key: role_key.clone(),
                    },
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocatedRoleAssignmentDatabaseCheckerError {
    /// The location where the error was encountered. This is the full path a field or a collection
    /// as well as the value.
    pub location: ErrorLocation,
    /// The encountered error.
    pub error: RoleAssignmentDatabaseCheckerError,
}

/// An enum of all of the errors that we may encounter when doing a database check. These errors are
/// collected and then returned at the end of database check.
#[derive(Debug, Clone)]
pub enum RoleAssignmentDatabaseCheckerError {
    /// A [`FieldIndex`] was encountered on a role-assignment module where the field index is not a
    /// valid [`RoleAssignmentField`].
    InvalidFieldIndex(FieldIndex),

    /// A [`CollectionIndex`] was encountered on a role-assignment module where the field index is
    /// not a valid [`RoleAssignmentCollection`].
    InvalidCollectionIndex(CollectionIndex),

    /// Attempted to decode the data as the FieldEntry associated with that field but failed to do
    /// so.
    FailedToDecodeFieldValue(RoleAssignmentField, Vec<u8>),

    /// Attempted to decode the data as the KeyPayload or EntryPayload associated with a collection
    /// entry but failed.
    FailedToDecodeCollectionKeyOrValue(RoleAssignmentCollection, Vec<u8>),

    /// An [`AccessRule`] was encountered which does not respect the width and depth limits.
    InvalidAccessRule(AccessRule, RoleAssignmentError),

    /// A reserved role-key was encountered.
    ReservedRoleKey(ModuleRoleKey),

    /// A role-key was encountered in a reserved space.
    RoleKeyInReservedSpace(ModuleRoleKey),

    /// A role-key was encountered which exceeds the maximum role-key length.
    RoleKeyExceedsMaximumLength {
        module_role_key: ModuleRoleKey,
        maximum_length: usize,
        actual: usize,
    },

    /// A role-key name contained disallowed characters
    RoleKeyNameCheckFailed(ModuleRoleKey, InvalidNameError),

    /// A role-key was encountered which is not in the list of initial role keys
    RoleKeyWasCreatedAfterInitialization {
        initial_role_keys: Vec<ModuleRoleKey>,
        role_key: ModuleRoleKey,
    },
}

#[derive(Debug, Clone)]
pub enum ErrorLocation {
    Field {
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        field_index: FieldIndex,
        value: Vec<u8>,
    },
    CollectionEntry {
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        collection_index: CollectionIndex,
        key: Vec<u8>,
        value: Vec<u8>,
    },
}
