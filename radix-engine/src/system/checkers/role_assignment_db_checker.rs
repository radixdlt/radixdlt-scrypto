use super::*;
use crate::blueprints::util::*;
use crate::internal_prelude::*;
use crate::object_modules::role_assignment::*;

#[derive(Debug, Clone, Default)]
pub struct RoleAssignmentDatabaseChecker {
    /// An optional vector of the initial role keys that the role-assignment module had been created
    /// with. An invariant that we have in the role-assignment module is that new role assignments
    /// can't be added after the creation of the role-assignment module. This check is done by the
    /// auth layer. If this is [`Some`], then the checker ensures that no role-keys are encountered
    /// of roles outside of the initial role-keys. If [`None`] then this check does not happen.
    initial_roles_keys: Option<Vec<ModuleRoleKey>>,

    /// The id of the node to check the role-assignment module of. If provided then all fields and
    /// collection entries belonging to other nodes are ignored.
    node_id: Option<NodeId>,

    /// A vector of all of the errors encountered when going through the RoleAssignment substates.
    errors: Vec<LocatedError<RoleAssignmentDatabaseCheckerError>>,
}

impl ApplicationChecker for RoleAssignmentDatabaseChecker {
    type ApplicationCheckerResults = Vec<LocatedError<RoleAssignmentDatabaseCheckerError>>;

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
            self.errors.push(LocatedError {
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

        let typed_field_index = RoleAssignmentField::from_repr(field_index)
            .expect("The application database checker does not check for this and assumes that other layers have checked for it.");

        match typed_field_index {
            RoleAssignmentField::Owner => {
                let owner_role = scrypto_decode::<RoleAssignmentOwnerFieldPayload>(&value)
                    .expect("The application database checker does not check for this and assumes that other layers have checked for it.");
                Self::check_owner_role_entry(owner_role, &mut add_error)
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
            self.errors.push(LocatedError {
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

        // Ignore all collection entries that do not belong to the role-assignment module.
        if module_id != ModuleId::RoleAssignment {
            return;
        }
        match self.node_id {
            Some(state_node_id) if state_node_id != node_id => return,
            _ => {}
        }

        let typed_collection_index = RoleAssignmentCollection::from_repr(collection_index)
            .expect("The application database checker does not check for this and assumes that other layers have checked for it.");

        match typed_collection_index {
            RoleAssignmentCollection::AccessRuleKeyValue => {
                let module_role_key = scrypto_decode::<RoleAssignmentAccessRuleKeyPayload>(&key)
                    .expect("The application database checker does not check for this and assumes that other layers have checked for it.");
                let access_rule = scrypto_decode::<RoleAssignmentAccessRuleEntryPayload>(&value)
                    .expect("The application database checker does not check for this and assumes that other layers have checked for it.");

                Self::check_role_assignment(
                    module_role_key,
                    access_rule,
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

    pub fn check_owner_role_entry<F>(
        owner_role_entry: RoleAssignmentOwnerFieldPayload,
        add_error: &mut F,
    ) where
        F: FnMut(RoleAssignmentDatabaseCheckerError),
    {
        let owner_rule = owner_role_entry
            .fully_update_and_into_latest_version()
            .owner_role_entry
            .rule;
        Self::check_access_rule_limits(owner_rule, add_error)
    }

    pub fn check_role_assignment<F>(
        key: RoleAssignmentAccessRuleKeyPayload,
        value: RoleAssignmentAccessRuleEntryPayload,
        initial_role_keys: &Option<Vec<ModuleRoleKey>>,
        add_error: &mut F,
    ) where
        F: FnMut(RoleAssignmentDatabaseCheckerError),
    {
        let key = key.content;
        let value = value.fully_update_and_into_latest_version();

        Self::check_access_rule_limits(value, add_error);
        Self::check_is_role_key_reserved(&key, add_error);
        Self::check_is_reserved_space(&key, add_error);
        Self::check_role_key_length(&key, add_error);
        Self::check_role_key_name(&key, add_error);
        Self::check_against_initial_role_keys(&key, initial_role_keys, add_error);
    }

    pub fn add_error(
        &mut self,
        location: ErrorLocation,
        error: RoleAssignmentDatabaseCheckerError,
    ) {
        self.errors.push(LocatedError { location, error })
    }

    fn check_access_rule_limits<F>(access_rule: AccessRule, add_error: &mut F)
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
        if RoleAssignmentNativePackage::is_role_key_reserved(&role_key.key) {
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

/// An enum of all of the errors that we may encounter when doing a database check. These errors are
/// collected and then returned at the end of database check.
#[derive(Debug, Clone)]
pub enum RoleAssignmentDatabaseCheckerError {
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
