use crate::blueprints::access_controller::v1::v1_0::AccessControllerV1Substate;
use crate::blueprints::access_controller::v1::*;
use crate::internal_prelude::*;
use crate::*;
use radix_blueprint_schema_init::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::*;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
#[sbor(type_name = "AccessControllerSubstate")]
pub struct AccessControllerV2Substate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: Vault,

    /// A vault that stores some XRD that can be used by any of the three roles for locking fees.
    pub xrd_fee_vault: Option<Vault>,

    /// The amount of time (in minutes) that it takes for timed recovery to be done. Maximum is
    /// 4,294,967,295 minutes which is 8171.5511700913 years. When this is [`None`], then timed
    /// recovery can not be performed through this access controller.
    pub timed_recovery_delay_in_minutes: Option<u32>,

    /// The resource address of the recovery badge that will be used by the wallet and optionally
    /// by other clients as well.
    pub recovery_badge: ResourceAddress,

    /// The states of the Access Controller.
    pub state: (
        // Controls whether the primary role is locked or unlocked
        PrimaryRoleLockingState,
        // Primary role recovery and withdraw states
        PrimaryRoleRecoveryAttemptState,
        PrimaryRoleBadgeWithdrawAttemptState,
        // Recovery role recovery and withdraw states
        RecoveryRoleRecoveryAttemptState,
        RecoveryRoleBadgeWithdrawAttemptState,
    ),
}

impl AccessControllerV2Substate {
    pub fn new(
        controlled_asset: Vault,
        xrd_fee_vault: Option<Vault>,
        timed_recovery_delay_in_minutes: Option<u32>,
        recovery_badge: ResourceAddress,
    ) -> Self {
        Self {
            controlled_asset,
            xrd_fee_vault,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state: Default::default(),
        }
    }
}

impl From<AccessControllerV1Substate> for AccessControllerV2Substate {
    fn from(
        AccessControllerV1Substate {
            controlled_asset,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state,
        }: AccessControllerV1Substate,
    ) -> Self {
        Self {
            controlled_asset,
            xrd_fee_vault: None,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state,
        }
    }
}

// TODO: Below are changes made to the expansion of the `declare_native_blueprint_state` since we
// do not have `StaticMultiVersioned` support yet. Once support for it is added then this will be
// moved to use that.

type AccessControllerStateV1 = AccessControllerV1Substate;
type AccessControllerStateV2 = AccessControllerV2Substate;

pub use access_controller_models::*;
#[allow(
    unused_imports,
    dead_code,
    unused_mut,
    unused_assignments,
    unused_variables,
    unreachable_code,
    unused_macros,
    unused_parens,
    clippy::result_unit_err
)]
mod access_controller_models {
    use super::*;
    use crate::errors::*;
    use crate::internal_prelude::*;
    use crate::system::system::*;
    use crate::track::interface::*;
    use radix_engine_interface::api::*;
    use sbor::*;
    macro_rules! VersionedAccessControllerState_trait_impl {
        ($trait:ty, $impl_block:tt) => {
            #[allow(dead_code)]impl $trait for VersionedAccessControllerState$impl_block
        };
    }
    #[allow(dead_code)]
    pub type AccessControllerState = AccessControllerStateV1;
    #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
    #[repr(u8)]
    pub enum VersionedAccessControllerState {
        V1(AccessControllerStateV1) = 1,
        V2(AccessControllerStateV2) = 2,
    }
    #[allow(dead_code)]
    impl VersionedAccessControllerState {
        pub fn new_latest(value: AccessControllerStateV2) -> Self {
            Self::V2(value)
        }
        pub fn update_once(self) -> sbor::UpdateResult<Self> {
            match self {
                Self::V1(value) => sbor::UpdateResult::Updated(Self::V2(value.into())),
                Self::V2(value) => sbor::UpdateResult::AtLatest(Self::V2(value)),
            }
        }
        pub fn update_to_latest(mut self) -> Self {
            loop {
                match self.update_once() {
                    sbor::UpdateResult::Updated(new) => {
                        self = new;
                    }
                    sbor::UpdateResult::AtLatest(latest) => {
                        return latest;
                    }
                }
            }
        }
    }
    #[allow(dead_code)]
    impl sbor::HasLatestVersion for VersionedAccessControllerState {
        type Latest = AccessControllerStateV2;
        #[allow(irrefutable_let_patterns)]
        fn into_latest(self) -> Self::Latest {
            let Self::V2(latest) = self.update_to_latest() else {
                panic!("Invalid resolved latest version not equal to latest type")
            };
            latest
        }
        #[allow(unreachable_patterns)]
        fn as_latest_ref(&self) -> Option<&Self::Latest> {
            match self {
                Self::V2(latest) => Some(latest),
                _ => None,
            }
        }
    }
    #[allow(dead_code)]
    impl From<AccessControllerStateV1> for VersionedAccessControllerState {
        fn from(value: AccessControllerStateV1) -> Self {
            Self::V1(value)
        }
    }
    #[allow(dead_code)]
    impl From<AccessControllerStateV2> for VersionedAccessControllerState {
        fn from(value: AccessControllerStateV2) -> Self {
            Self::V2(value)
        }
    }
    #[allow(dead_code)]
    pub trait VersionedAccessControllerStateVersion {
        type Versioned;
        fn into_versioned(self) -> Self::Versioned;
    }
    macro_rules! VersionedAccessControllerState_versionable_impl {
        ($inner_type:ty) => {
            impl VersionedAccessControllerStateVersion for$inner_type {
                type Versioned = VersionedAccessControllerState;
                fn into_versioned(self)->Self::Versioned {
                    self.into()
                }
            }
        };
    }
    impl VersionedAccessControllerStateVersion for AccessControllerStateV1 {
        type Versioned = VersionedAccessControllerState;
        fn into_versioned(self) -> Self::Versioned {
            self.into()
        }
    }
    #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
    #[sbor(transparent, categorize_types = "")]
    /// This new type represents the payload of a particular field or collection.
    /// It is unique to this particular field/collection.
    pub struct AccessControllerStateFieldPayload {
        pub content: VersionedAccessControllerState,
    }
    impl core::convert::From<VersionedAccessControllerState> for AccessControllerStateFieldPayload {
        fn from(value: VersionedAccessControllerState) -> Self {
            Self { content: value }
        }
    }
    impl core::convert::AsRef<VersionedAccessControllerState> for AccessControllerStateFieldPayload {
        fn as_ref(&self) -> &VersionedAccessControllerState {
            &self.content
        }
    }
    impl core::convert::AsMut<VersionedAccessControllerState> for AccessControllerStateFieldPayload {
        fn as_mut(&mut self) -> &mut VersionedAccessControllerState {
            &mut self.content
        }
    }
    impl FieldPayload for AccessControllerStateFieldPayload {
        type Content = VersionedAccessControllerState;
        fn into_content(self) -> Self::Content {
            self.content
        }
    }
    impl FieldContentSource<AccessControllerStateFieldPayload> for VersionedAccessControllerState {
        fn into_content(self) -> VersionedAccessControllerState {
            self
        }
    }
    impl HasLatestVersion for AccessControllerStateFieldPayload {
        type Latest = <VersionedAccessControllerState as HasLatestVersion>::Latest;
        fn into_latest(self) -> Self::Latest {
            self.into_content().into_latest()
        }
        fn as_latest_ref(&self) -> Option<&Self::Latest> {
            self.as_ref().as_latest_ref()
        }
    }
    impl FieldContentSource<AccessControllerStateFieldPayload> for AccessControllerState {
        fn into_content(self) -> VersionedAccessControllerState {
            self.into()
        }
    }
    pub type AccessControllerStateFieldSubstate =
        crate::system::system_substates::FieldSubstate<AccessControllerStateFieldPayload>;
    pub struct AccessControllerStateSchemaInit;

    impl AccessControllerStateSchemaInit {
        pub fn create_schema_init(
            type_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>,
        ) -> BlueprintStateSchemaInit {
            let mut fields = (sbor::rust::vec::Vec::new());
            fields.push(FieldSchema {
                field: TypeRef::Static(
                    type_aggregator
                        .add_child_type_and_descendents::<AccessControllerStateFieldPayload>(),
                ),
                condition: { (Condition::Always) },
                transience: { FieldTransience::NotTransient },
            });
            let mut collections = (sbor::rust::vec::Vec::new());
            BlueprintStateSchemaInit {
                fields,
                collections,
            }
        }
    }
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
    pub enum AccessControllerField {
        State,
    }
    impl AccessControllerField {
        pub const fn field_index(&self) -> u8 {
            *self as u8
        }
    }
    impl From<AccessControllerField> for SubstateKey {
        fn from(value: AccessControllerField) -> Self {
            SubstateKey::Field(value as u8)
        }
    }
    impl From<AccessControllerField> for u8 {
        fn from(value: AccessControllerField) -> Self {
            value as u8
        }
    }
    impl TryFrom<&SubstateKey> for AccessControllerField {
        type Error = ();
        fn try_from(key: &SubstateKey) -> Result<Self, Self::Error> {
            match key {
                SubstateKey::Field(x) => Self::from_repr(*x).ok_or(()),
                _ => Err(()),
            }
        }
    }
    impl TryFrom<u8> for AccessControllerField {
        type Error = ();
        fn try_from(offset: u8) -> Result<Self, Self::Error> {
            Self::from_repr(offset).ok_or(())
        }
    }
    impl FieldDescriptor for AccessControllerField {
        fn field_index(&self) -> FieldIndex {
            *self as u8
        }
    }
    #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum AccessControllerCollection {}

    impl CollectionDescriptor for AccessControllerCollection {
        fn collection_index(&self) -> CollectionIndex {
            unreachable!()
        }
    }
    #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash)]
    pub enum AccessControllerFeature {}

    impl BlueprintFeature for AccessControllerFeature {
        fn feature_name(&self) -> &'static str {
            unreachable!()
        }
    }
    #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, Default)]
    pub struct AccessControllerFeatureSet {}

    impl AccessControllerFeatureSet {
        pub fn all_features() -> IndexSet<String> {
            Default::default()
        }
    }
    impl HasFeatures for AccessControllerFeatureSet {
        fn feature_names_str(&self) -> Vec<&'static str> {
            Default::default()
        }
    }
    /// All the SubstateKeys for all logical partitions for the $blueprint_ident blueprint.
    /// Does not include mapped partitions, as these substates are mapped via their canonical
    /// partition.
    #[derive(Debug, Clone)]
    pub enum AccessControllerTypedSubstateKey {
        Field(AccessControllerField),
    }
    impl AccessControllerTypedSubstateKey {
        pub fn for_key_at_partition_offset(
            partition_offset: PartitionOffset,
            substate_key: &SubstateKey,
        ) -> Result<Self, ()> {
            Self::for_key_in_partition(
                &AccessControllerPartitionOffset::try_from(partition_offset)?,
                substate_key,
            )
        }
        pub fn for_key_in_partition(
            partition: &AccessControllerPartitionOffset,
            substate_key: &SubstateKey,
        ) -> Result<Self, ()> {
            let key = match partition {
                AccessControllerPartitionOffset::Field => AccessControllerTypedSubstateKey::Field(
                    AccessControllerField::try_from(substate_key)?,
                ),
            };
            Ok(key)
        }
    }
    #[derive(Debug)]
    pub enum AccessControllerTypedFieldSubstateValue {
        State(AccessControllerStateFieldSubstate),
    }
    /// All the Substate values for all logical partitions for the $blueprint_ident blueprint.
    /// Does not include mapped partitions, as these substates are mapped via their canonical partition.
    #[derive(Debug)]
    pub enum AccessControllerTypedSubstateValue {
        Field(AccessControllerTypedFieldSubstateValue),
    }
    impl AccessControllerTypedSubstateValue {
        pub fn from_key_and_data(
            key: &AccessControllerTypedSubstateKey,
            data: &[u8],
        ) -> Result<Self, DecodeError> {
            let substate_value = match key {
                AccessControllerTypedSubstateKey::Field(AccessControllerField::State) => {
                    AccessControllerTypedSubstateValue::Field(
                        AccessControllerTypedFieldSubstateValue::State(scrypto_decode(data)?),
                    )
                }
            };
            Ok(substate_value)
        }
    }
}
