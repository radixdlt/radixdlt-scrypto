use crate::errors::*;
use crate::system::system::*;
use crate::types::*;

// TODO(David) - Move this, features and content stuff under a `models` module?
// TODO(David) - Create an internal_prelude in `blueprints`
// and an internal_prelude and prelud in engine
pub trait FeatureSetResolver: Debug {
    fn feature_names_str(&self) -> Vec<&'static str>;

    fn feature_names_str_set(&self) -> BTreeSet<&'static str> {
        self.feature_names_str().into_iter().collect()
    }

    fn feature_names_string(&self) -> Vec<String> {
        self.feature_names_str()
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }

    fn feature_names_string_set(&self) -> BTreeSet<String> {
        self.feature_names_str()
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }
}

/// For feature checks against a non-inner object
#[derive(Debug)]
pub enum FeatureChecks<TOwn: FeatureSetResolver> {
    None,
    RequireAllSubstates,
    ForFeatures { own_features: TOwn },
}

impl<T: FeatureSetResolver> From<T> for FeatureChecks<T> {
    fn from(value: T) -> Self {
        FeatureChecks::ForFeatures {
            own_features: value,
        }
    }
}

impl<TOwn: FeatureSetResolver> FeatureChecks<TOwn> {
    pub fn assert_valid(
        &self,
        substate_name: &'static str,
        condition: &Condition,
        is_present: bool,
    ) -> Result<(), RuntimeError> {
        let is_valid = match self {
            FeatureChecks::None => Ok(()),
            FeatureChecks::RequireAllSubstates => {
                if is_present {
                    Ok(())
                } else {
                    Err(format!("Required all substates to be present, but {} was not present", substate_name))
                }
            },
            FeatureChecks::ForFeatures { own_features } => {
                match condition {
                    Condition::Always => {
                        if is_present {
                            Ok(())
                        } else {
                            Err(format!("Substate condition for {} required it to be always present, but it was not", substate_name))
                        }
                    }
                    Condition::IfFeature(feature) => {
                        let feature_enabled = own_features.feature_names_str().contains(&feature.as_str());
                        if feature_enabled && !is_present {
                            Err(format!("Substate condition for {} required it to be present when the feature {} was enabled, but it was absent", substate_name, feature))
                        } else if !feature_enabled && is_present {
                            Err(format!("Substate condition for {} required it to be absent when the feature {} was disabled, but it was present", substate_name, feature))
                        } else {
                            Ok(())
                        }
                    },
                    Condition::IfOuterFeature(_) => {
                        Err(format!("Substate condition for {} required an outer object feature, but the blueprint does not have an outer blueprint defined", substate_name))
                    }
                }
            },
        };
        is_valid.map_err(|error_message| {
            RuntimeError::SystemError(SystemError::InvalidNativeSubstatesForFeature(error_message))
        })
    }
}

/// For feature checks against an inner object
pub enum InnerObjectFeatureChecks<TOwn, TOuter> {
    None,
    RequireAllSubstates,
    ForFeatures {
        own_features: TOwn,
        outer_object_features: TOuter,
    },
}

impl<TOwn: FeatureSetResolver, TOuter: FeatureSetResolver> InnerObjectFeatureChecks<TOwn, TOuter> {
    pub fn assert_valid(
        &self,
        substate_name: &'static str,
        condition: &Condition,
        is_present: bool,
    ) -> Result<(), RuntimeError> {
        let is_valid = match self {
            Self::None => Ok(()),
            Self::RequireAllSubstates => {
                if is_present {
                    Ok(())
                } else {
                    Err(format!(
                        "Required all substates to be present, but {} was not present",
                        substate_name
                    ))
                }
            }
            Self::ForFeatures {
                own_features,
                outer_object_features,
            } => match condition {
                Condition::Always => {
                    if is_present {
                        Ok(())
                    } else {
                        Err(format!("Substate condition for {} required it to be always present, but it was not", substate_name))
                    }
                }
                Condition::IfFeature(feature) => {
                    let feature_enabled =
                        own_features.feature_names_str().contains(&feature.as_str());
                    if feature_enabled && !is_present {
                        Err(format!("Substate condition for {} required it to be present when the feature {} was enabled, but it was absent", substate_name, feature))
                    } else if !feature_enabled && is_present {
                        Err(format!("Substate condition for {} required it to be absent when the feature {} was disabled, but it was present", substate_name, feature))
                    } else {
                        Ok(())
                    }
                }
                Condition::IfOuterFeature(feature) => {
                    let feature_enabled = outer_object_features
                        .feature_names_str()
                        .contains(&feature.as_str());
                    if feature_enabled && !is_present {
                        Err(format!("Substate condition for {} required it to be present when the outer object feature {} was enabled, but it was absent", substate_name, feature))
                    } else if !feature_enabled && is_present {
                        Err(format!("Substate condition for {} required it to be absent when the outer object feature {} was disabled, but it was present", substate_name, feature))
                    } else {
                        Ok(())
                    }
                }
            },
        };
        is_valid.map_err(|error_message| {
            RuntimeError::SystemError(SystemError::InvalidNativeSubstatesForFeature(error_message))
        })
    }
}

pub trait FieldContent<FieldPayload: From<Self>>: Sized {
    fn into_locked_substate(self) -> FieldSubstate<FieldPayload> {
        FieldSubstate::new_locked_field(self.into())
    }

    fn into_mutable_substate(self) -> FieldSubstate<FieldPayload> {
        FieldSubstate::new_field(self.into())
    }
}

pub trait KeyContent<KeyPayload: From<Self>>: Sized {
    fn into_key(self) -> KeyPayload {
        self.into()
    }
}

pub trait KeyValueEntryContent<EntryPayload: From<Self>>: Sized {
    fn into_locked_substate(self) -> KeyValueEntrySubstate<EntryPayload> {
        KeyValueEntrySubstate::entry(self.into())
    }

    fn into_mutable_substate(self) -> KeyValueEntrySubstate<EntryPayload> {
        KeyValueEntrySubstate::locked_entry(self.into())
    }
}

pub trait IndexEntryContent<EntryPayload: From<Self>>: Sized {
    fn into_substate(self) -> EntryPayload {
        self.into()
    }
}

pub trait SortedIndexEntryContent<EntryPayload: From<Self>>: Sized {
    fn into_substate(self) -> EntryPayload {
        self.into()
    }
}

/// Generates types and typed-interfaces for native blueprints, their
/// state models, features, partitions, and their interaction with
/// the substate store.
///
/// See the below structure for detail on how it should look - or check
/// out [../package/substates.rs](the package substates definition).
///
/// For each field, the following types will be created:
/// * `<BlueprintIdent><FieldIdent>FieldPayload` - a transparent new type for the field content
/// * `<BlueprintIdent><FieldIdent>FieldSubstate` - a type for the full system-wrapped substate
///
/// For each collection value, the following types will be created:
/// * `<BlueprintIdent><CollectionIdent>EntryPayload` - a transparent new type for the entry content
/// * `<BlueprintIdent><CollectionIdent>EntrySubstate` - a type for the full system-wrapped substate
///
/// For each collection key, the following types will be created:
/// * `<BlueprintIdent><CollectionIdent>KeyPayload` - a transparent new type for the key content
/// * `<BlueprintIdent><CollectionIdent>SubstateKey` - a type for the full key (eg includes the u16 for a sorted index key)
///
/// The content of each of the above can take a number of forms.
/// This is configured via specifying the type as one of the following.
/// By default, choose StaticSingleVersioned for fields and collection values.
/// ```
///     {
///         kind: StaticSingleVersioned,
///     }
///     {
///         kind: Static,
///         the_type: x,
///     },
///     {
///         kind: Instance,
///         ident: GenericTypeParameterName,
///     },
///     // In future
///     {
///         kind: StaticMultiVersioned,
///         previous_versions: [V1, V2],
///         latest: V3,
///     }
/// ```
///
/// Choosing  `StaticSingleVersioned`, which will create a
/// forward-compatible enum wrapper with a single version for the content.
/// For Fields, it will assume the existence of a type called
/// `<BlueprintIdent><FieldIdent>V1` and will generate the following types:
/// * `<BlueprintIdent><FieldIdent>` - a type alias for the latest version (V1).
/// * `Versioned<BlueprintIdent><FieldIdent>` - the enum wrapper with a single version. This will be the content of `<BlueprintIdent><FieldIdent>FieldPayload`.
///
/// For collection values, it will assume the existence of `<BlueprintIdent><CollectionIdent>V1`
/// and generate the following types:
/// * `<BlueprintIdent><CollectionIdent>` - a type alias for the latest version (V1).
/// * `Versioned<BlueprintIdent><CollectionIdent>` - the enum wrapper with a single version. This will be the content of `<BlueprintIdent><CollectionIdent>EntryPayload`.
///
/// For collection keys, it will assume the existence of `<BlueprintIdent><CollectionIdent>KeyInnerV1`
/// and generate the following types:
/// * `<BlueprintIdent><CollectionIdent>KeyInner` - a type alias for the latest version (V1)
/// * `Versioned<BlueprintIdent><CollectionIdent>KeyInner` - the enum wrapper with a single version. This will be the content of `<BlueprintIdent><CollectionIdent>KeyPayload`.
#[allow(unused)]
macro_rules! declare_native_blueprint_state {
    (
        blueprint_ident: $blueprint_ident:ident,
        blueprint_snake_case: $blueprint_property_name:ident,
        $(
            outer_blueprint: {
                ident: $outer_blueprint_ident:ident
                $(,)?
            },
        )?
        features: {
            $(
                $feature_property_name:ident: {
                    ident: $feature_ident:ident,
                    description: $feature_description:expr,
                }
            ),*
            $(,)?
        },
        instance_schema_types: {
            // If no types => instance schema disabled
            $(
                $instance_type_property_name:ident: {
                    ident: $instance_type_ident:ident,
                }
            ),*
            $(,)?
        },
        fields: {
            $(
                $field_property_name:ident: {
                    ident: $field_ident:ident,
                    field_type: $field_type:tt,
                    condition: $field_condition:expr
                    $(,)? // Optional trailing comma
                }
            ),*
            $(,)? // Optional trailing comma
        },
        collections: {
            $(
                $collection_property_name:ident: $collection_type:ident {
                    entry_ident: $collection_ident:ident,
                    key_type: $collection_key_type:tt,
                    value_type: $collection_value_type:tt,
                    can_own: $collection_can_own:expr
                    // Collection options for (eg) passing in a property name
                    // of the sorted index parameter for SortedIndex
                    $(, options: $collection_options:tt)?
                    $(,)? // Optional trailing comma
                }
            ),*
            $(,)? // Optional trailing comma
        }
        $(,)?
    ) => {
        paste::paste! {
            pub use [<$blueprint_property_name _models>]::*;

            #[allow(unused_imports, dead_code, unused_mut, unused_assignments, unused_variables, unreachable_code)]
            mod [<$blueprint_property_name _models>] {
                use super::*;
                use sbor::*;
                use $crate::types::*;
                use $crate::track::interface::*;
                use $crate::errors::*;
                use $crate::system::system::*;
                use radix_engine_interface::api::*;
                //--------------------------------------------------------
                // MODELS
                //--------------------------------------------------------

                // Generate models for each field
                $(
                    // Value
                    // > Set up Versioned types (if relevant). Assumes __FieldV1 exists and then creates
                    //   - Versioned__Field
                    //   - __Field (alias for __FieldV1)
                    // > Set up the (transparent) _FieldPayload new type for the content of the field
                    // > Set up the FieldContent trait for anything which can be resolved into the field payload
                    generate_content_type!(
                        content_trait: FieldContent,
                        ident_core: [<$blueprint_ident $field_ident>],
                        #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
                        struct [<$blueprint_ident $field_ident FieldPayload>] = $field_type
                    );

                    // > Set up the _FieldSubstate alias for the system-wrapped substate
                    generate_system_substate_type_alias!(
                        Field,
                        type [<$blueprint_ident $field_ident FieldSubstate>] = WRAPPED [<$blueprint_ident $field_ident FieldPayload>]
                    );
                );*

                // Generate models for each collection
                $(
                    // Key
                    // > Set up Versioned types (if relevant). Assumes __KeyInnerV1 exists and then creates
                    //   - Versioned__KeyInner
                    //   - __KeyInner (alias for __KeyInnerV1)
                    // > Create the (transparent) _Key new type for the key
                    // > Set up the KeyContent traits for anything which can be resolved into a key
                    generate_content_type!(
                        content_trait: KeyContent,
                        ident_core: [<$blueprint_ident $collection_ident KeyInner>],
                        #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor)]
                        #[sbor(transparent_name)]
                        struct [<$blueprint_ident $collection_ident KeyPayload>] = $collection_key_type
                    );

                    // TODO(David): Tweak when I work out the right strategy for keys. Probably just want a single new-type.
                    // So probably just don't use generate_content_type but use generate_key_type or something instead?
                    pub type [<$blueprint_ident $collection_ident SubstateKey>] = [<$blueprint_ident $collection_ident KeyPayload>];

                    // TODO(David) - Properly handle SortedIndex:
                    // Fix Key types for SortedIndex to have a named u16 part of the key,
                    // use a different key trait, and use .for_sorted_key in the below.
                    impl TryFrom<&SubstateKey> for [<$blueprint_ident $collection_ident SubstateKey>] {
                        type Error = ();

                        fn try_from(substate_key: &SubstateKey) -> Result<Self, Self::Error> {
                            let key = substate_key.for_map().ok_or(())?;
                            scrypto_decode::<Self>(&key).map_err(|_| ())
                        }
                    }

                    // Values
                    // > Set up Versioned types (if relevant). Assumes __ValueV1 exists and then creates
                    //   - Versioned__Value
                    //   - __Value (alias for __ValueV1)
                    // > Set up the (transparent) _ValueContent new type for the value content
                    // > Set up the _EntryContent traits
                    generate_content_type!(
                        content_trait: [<$collection_type EntryContent>],
                        ident_core: [<$blueprint_ident $collection_ident>],
                        #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
                        struct [<$blueprint_ident $collection_ident EntryPayload>] = $collection_value_type
                    );
                    // > Set up the _EntrySubstate alias for the system-wrapped substate
                    generate_system_substate_type_alias!(
                        $collection_type,
                        type [<$blueprint_ident $collection_ident EntrySubstate>] = WRAPPED [<$blueprint_ident $collection_ident EntryPayload>]
                    );
                )*

                //--------------------------------------------------------
                // Node Layout (to replace node_layout.rs)
                //--------------------------------------------------------
                #[repr(u8)]
                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
                pub enum [<$blueprint_ident Field>] {
                    $($field_ident,)*
                }

                impl From<[<$blueprint_ident Field>]> for SubstateKey {
                    fn from(value: [<$blueprint_ident Field>]) -> Self {
                        SubstateKey::Field(value as u8)
                    }
                }

                impl From<[<$blueprint_ident Field>]> for u8 {
                    fn from(value: [<$blueprint_ident Field>]) -> Self {
                        value as u8
                    }
                }

                impl TryFrom<&SubstateKey> for [<$blueprint_ident Field>] {
                    type Error = ();

                    fn try_from(key: &SubstateKey) -> Result<Self, Self::Error> {
                        match key {
                            SubstateKey::Field(x) => Self::from_repr(*x).ok_or(()),
                            _ => Err(()),
                        }
                    }
                }

                impl TryFrom<u8> for [<$blueprint_ident Field>] {
                    type Error = ();

                    fn try_from(offset: u8) -> Result<Self, Self::Error> {
                        Self::from_repr(offset).ok_or(())
                    }
                }

                #[repr(u8)]
                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
                pub enum [<$blueprint_ident Partition>] {
                    Field,
                    $([<$collection_ident $collection_type>],)*
                }

                impl [<$blueprint_ident Partition>] {
                    pub const fn offset(&self) -> PartitionOffset {
                        PartitionOffset(*self as u8)
                    }

                    pub const fn as_main_partition(&self) -> PartitionNumber {
                        match MAIN_BASE_PARTITION.at_offset(self.offset()) {
                            // Work around .unwrap() on Option not being const
                            Some(x) => x,
                            None => panic!("Offset larger than allowed value")
                        }
                    }
                }

                impl From<[<$blueprint_ident Partition>]> for PartitionOffset {
                    fn from(value: [<$blueprint_ident Partition>]) -> Self {
                        PartitionOffset(value as u8)
                    }
                }

                impl TryFrom<PartitionOffset> for [<$blueprint_ident Partition>] {
                    type Error = ();

                    fn try_from(offset: PartitionOffset) -> Result<Self, Self::Error> {
                        Self::from_repr(offset.0).ok_or(())
                    }
                }

                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash)]
                pub struct [<$blueprint_ident FeatureSet>] {
                    $(pub [<$feature_property_name>]: bool,)*
                }

                impl FeatureSetResolver for [<$blueprint_ident FeatureSet>] {
                    fn feature_names_str(&self) -> Vec<&'static str> {
                        let mut names = vec![];
                        $(
                            if self.[<$feature_property_name>] {
                                names.push(stringify!($feature_property_name));
                            }
                        )*
                        names
                    }
                }

                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash)]
                pub enum [<$blueprint_ident Feature>] {
                    $($feature_ident,)*
                }

                impl BlueprintFeature for [<$blueprint_ident Feature>] {
                    fn feature_name(&self) -> &'static str {
                        match *self {
                            $(
                                Self::$feature_ident => stringify!($feature_property_name),
                            )*
                        }
                    }
                }

                //---------------------------------
                // Typed Substate - Keys and Values
                //---------------------------------
                #[derive(Debug, Clone)]
                pub enum [<$blueprint_ident TypedSubstateKey>] {
                    Fields([<$blueprint_ident Field>]),
                    $([<$collection_ident $collection_type Entries>]([<$blueprint_ident $collection_ident SubstateKey>]),)*
                }

                impl [<$blueprint_ident TypedSubstateKey>] {
                    pub fn for_key_in_partition(partition: &[<$blueprint_ident Partition>], substate_key: &SubstateKey) -> Result<Self, ()> {
                        let key = match partition {
                            [<$blueprint_ident Partition>]::Field => {
                                [<$blueprint_ident TypedSubstateKey>]::Fields(
                                    [<$blueprint_ident Field>]::try_from(substate_key)?
                                )
                            }
                            $(
                                [<$blueprint_ident Partition>]::[<$collection_ident $collection_type>] => {
                                    [<$blueprint_ident TypedSubstateKey>]::[<$collection_ident $collection_type Entries>](
                                        [<$blueprint_ident $collection_ident SubstateKey>]::try_from(substate_key)?,
                                    )
                                }
                            )*
                        };
                        Ok(key)
                    }
                }

                #[derive(Debug)]
                pub enum [<$blueprint_ident TypedFieldSubstateValue>] {
                    $($field_ident([<$blueprint_ident $field_ident FieldSubstate>]),)*
                }

                #[derive(Debug)]
                pub enum [<$blueprint_ident TypedSubstateValue>] {
                    Field([<$blueprint_ident TypedFieldSubstateValue>]),
                    $([<$collection_ident $collection_type>]([<$blueprint_ident $collection_ident EntrySubstate>]),)*
                }

                impl [<$blueprint_ident TypedSubstateValue>] {
                    pub fn from_key_and_data(key: &[<$blueprint_ident TypedSubstateKey>], data: &[u8]) -> Result<Self, DecodeError> {
                        let substate_value = match key {
                            // Fields
                            $(
                                [<$blueprint_ident TypedSubstateKey>]::Fields([<$blueprint_ident Field>]::$field_ident) => {
                                    [<$blueprint_ident TypedSubstateValue>]::Field(
                                        [<$blueprint_ident TypedFieldSubstateValue>]::$field_ident(scrypto_decode(data)?)
                                    )
                                }
                            )*
                            // Collections
                            $(
                                [<$blueprint_ident TypedSubstateKey>]::[<$collection_ident $collection_type Entries>](_) => {
                                    [<$blueprint_ident TypedSubstateValue>]::[<$collection_ident $collection_type>](
                                        scrypto_decode(data)?
                                    )
                                }
                            )*
                        };
                        Ok(substate_value)
                    }
                }

                //----------------------
                // Schema
                //----------------------
                pub struct [<$blueprint_ident StateSchemaInit>];

                impl [<$blueprint_ident StateSchemaInit>] {
                    pub fn create_schema_init(
                        type_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>,
                    ) -> BlueprintStateSchemaInit {
                        let mut fields = vec![];
                        $(
                            // TODO(David) - Implement instance schema
                            fields.push(FieldSchema {
                                field: TypeRef::Static(
                                    type_aggregator.add_child_type_and_descendents::<[<$blueprint_ident $field_ident FieldPayload>]>()
                                ),
                                condition: $field_condition,
                            });
                        )*
                        let mut collections = vec![];
                        $(
                            // TODO(David) - Implement instance schema
                            collections.push(map_collection_schema!(
                                $collection_type,
                                type_aggregator,
                                $collection_key_type,
                                [<$blueprint_ident $collection_ident KeyPayload>],
                                $collection_value_type,
                                [<$blueprint_ident $collection_ident EntryPayload>],
                                $collection_can_own
                            ));
                        )*
                        BlueprintStateSchemaInit {
                            fields,
                            collections,
                        }
                    }
                }

                //----------------------
                // Object Initialization
                //----------------------

                /// Used for initializing blueprint state.
                ///
                /// Note - this doesn't support:
                /// * Features
                /// * Instance schemas
                /// * Feature-dependent fields
                /// * IndexEntries (because the underlying new_object API doesn't support them)
                #[derive(Debug, Default)]
                pub struct [<$blueprint_ident StateInit>] {
                    $(
                        pub $field_property_name: Option<[<$blueprint_ident $field_ident FieldSubstate>]>,
                    )*
                    $(
                        pub $collection_property_name: IndexMap<
                            [<$blueprint_ident $collection_ident SubstateKey>],
                            [<$blueprint_ident $collection_ident EntrySubstate>]
                        >,
                    )*
                }

                type [<$blueprint_ident FeatureChecks>] = [<$(ignore_arg!($outer_blueprint_ident) InnerObject)? FeatureChecks>]::<
                    [<$blueprint_ident FeatureSet>],
                    $([<$outer_blueprint_ident FeatureSet>],)?
                >;

                impl [<$blueprint_ident StateInit>] {
                    pub fn into_system_substates(self, feature_checks: [<$blueprint_ident FeatureChecks>]) -> Result<(BTreeMap<u8, FieldValue>, BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>), RuntimeError> {
                        let mut field_values = BTreeMap::new();
                        $(
                            {
                                feature_checks.assert_valid(
                                    stringify!($field_ident),
                                    &$field_condition,
                                    self.$field_property_name.is_some(),
                                )?;
                                if let Some(field) = self.$field_property_name {
                                    let field_content = scrypto_encode(&field.field_content()).unwrap();
                                    let locked = match &field.mutability {
                                        SubstateMutability::Mutable => true,
                                        SubstateMutability::Immutable => false,
                                    };
                                    field_values.insert(
                                        [<$blueprint_ident Field>]::$field_ident.into(),
                                        FieldValue {
                                            value: field_content,
                                            locked,
                                        }
                                    );
                                }
                            }
                        )*
                        let mut all_collection_entries = BTreeMap::new();
                        let mut collection_index: u8 = 0;
                        $(
                            {
                                let this_collection_entries = self.$collection_property_name
                                    .into_iter()
                                    .map(|(key, entry_substate)| {
                                        let key = scrypto_encode(&key).unwrap();
                                        let value = map_entry_substate_to_kv_entry!($collection_type, entry_substate);
                                        (key, value)
                                    })
                                    .collect();
                                all_collection_entries.insert(collection_index, this_collection_entries);
                                collection_index += 1;
                            }
                        )*
                        Ok((field_values, all_collection_entries))
                    }

                    // TODO: Remove this when the new object api supports non-vec field values
                    pub fn into_system_substates_legacy(self, feature_checks: [<$blueprint_ident FeatureChecks>]) -> Result<(Vec<FieldValue>, BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>), RuntimeError> {
                        let (mut field_values, collection_entries) = self.into_system_substates(feature_checks)?;
                        let mut field_values_vec = vec![];
                        let mut field_index = 0;
                        $(
                            {
                                let field_value = field_values.remove(&field_index).expect(
                                    concat!(
                                        "The field `",
                                        stringify!($field_property_name),
                                        "` was None. Until the system and macro supports feature-based optional fields, all fields need to be present"
                                    )
                                );
                                field_values_vec.push(field_value);
                                field_index += 1;
                            }
                        )*
                        Ok((field_values_vec, collection_entries))
                    }

                    /// This is used mostly for flashing
                    pub fn into_kernel_main_partitions(self, feature_checks: [<$blueprint_ident FeatureChecks>]) -> Result<NodeSubstates, RuntimeError> {
                        // PartitionNumber => SubstateKey => IndexedScryptoValue
                        let mut partitions: NodeSubstates = BTreeMap::new();
                        let (mut field_values, mut kv_entries) = self.into_system_substates(feature_checks)?;

                        // Fields
                        {
                            let mut field_partition_substates = BTreeMap::new();
                            for (field_index, field_value) in field_values {
                                field_partition_substates.insert(
                                    SubstateKey::Field(field_index),
                                    IndexedScryptoValue::from_typed(&field_value),
                                );
                            }
                            partitions.insert(
                                [<$blueprint_ident Partition>]::Field.as_main_partition(),
                                field_partition_substates,
                            );
                        }

                        // Each Collection
                        let mut collection_index = 0u8;
                        $({
                            let collection_kv_entries = kv_entries.remove(&collection_index).unwrap();
                            let collection_partition = [<$blueprint_ident Partition>]::[<$collection_ident $collection_type>];
                            let collection_partition_substates = collection_kv_entries
                                .into_iter()
                                .filter_map(|(key_bytes, kv_entry)| {
                                    let substate = match kv_entry {
                                        KVEntry { value: Some(value_bytes), locked: false } => {
                                            KeyValueEntrySubstate::entry(
                                                scrypto_decode::<ScryptoValue>(&value_bytes).unwrap()
                                            )
                                        }
                                        KVEntry { value: Some(value_bytes), locked: true } => {
                                            KeyValueEntrySubstate::locked_entry(
                                                scrypto_decode::<ScryptoValue>(&value_bytes).unwrap()
                                            )
                                        }
                                        KVEntry { value: None, locked: true } => {
                                            KeyValueEntrySubstate::locked_empty_entry()
                                        }
                                        KVEntry { value: None, locked: false } => {
                                            return None;
                                        }
                                    };

                                    Some((
                                        SubstateKey::Map(key_bytes),
                                        IndexedScryptoValue::from_typed(&substate)
                                    ))
                                })
                                .collect();
                            partitions.insert(
                                collection_partition.as_main_partition(),
                                collection_partition_substates,
                            );
                            collection_index += 1;
                        })*

                        Ok(partitions)
                    }

                    pub fn into_new_object<Y: ClientObjectApi<RuntimeError>>(
                        self,
                        api: &mut Y,
                        own_features: [<$blueprint_ident FeatureSet>],
                        $(outer_object_features: [<$outer_blueprint_ident FeatureSet>],)?
                        instance_schema: Option<InstanceSchemaInit>,
                    ) -> Result<NodeId, RuntimeError> {
                        let (field_values, all_collection_entries) = self.into_system_substates_legacy(
                            [<$blueprint_ident FeatureChecks>]::ForFeatures {
                                own_features,
                                $(ignore_arg!($outer_blueprint_ident) outer_object_features,)?
                            }
                        )?;
                        api.new_object(
                            stringify!($blueprint_ident),
                            own_features.feature_names_str(),
                            instance_schema,
                            field_values, // TODO: Change to take the IndexMap and get rid of into_system_substates_legacy
                            all_collection_entries, // TODO: Change to take other collections, not just KVEntry
                        )
                    }
                }

                //-------------
                // State API (TODO!)
                //-------------

                pub struct [<$blueprint_ident StateApi>]<'a, Y: ClientApi<RuntimeError>> {
                    api: &'a mut Y,
                }

                impl<'a, Y: ClientApi<RuntimeError>> [<$blueprint_ident StateApi>]<'a, Y> {
                    pub fn with(client_api: &'a mut Y) -> Self {
                        Self {
                            api: client_api,
                        }
                    }
                }

                impl<'a, Y: ClientApi<$crate::errors::RuntimeError>> From<&'a mut Y> for [<$blueprint_ident StateApi>]<'a, Y> {
                    fn from(value: &'a mut Y) -> Self {
                        Self::with(value)
                    }
                }
            }
        }
    }
}

#[allow(unused)]
pub(crate) use declare_native_blueprint_state;

pub(crate) use helper_macros::*;

#[allow(unused_macros)]
mod helper_macros {
    macro_rules! ignore_arg {
        ($($ignored:tt)*) => {};
    }
    #[allow(unused)]
    pub(crate) use ignore_arg;

    macro_rules! generate_content_type {
        (
            content_trait: $content_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $content_type_name:ident = {
                kind: StaticSingleVersioned
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                sbor::define_single_versioned!(
                    $(#[$attributes])*
                    pub enum [<Versioned $ident_core>] => $ident_core = [<$ident_core V1>]
                );
                $(#[$attributes])*
                #[sbor(transparent)]
                pub struct $content_type_name(pub [<Versioned $ident_core>]);
                impl From<[<Versioned $ident_core>]> for $content_type_name {
                    fn from(value: [<Versioned $ident_core>]) -> Self {
                        Self(value)
                    }
                }
                impl $content_trait<$content_type_name> for [<Versioned $ident_core>] {}
                // Also add impls from the "latest" type
                impl From<$ident_core> for $content_type_name {
                    fn from(value: $ident_core) -> Self {
                        Self(value.into())
                    }
                }
                impl $content_trait<$content_type_name> for $ident_core {}
            }
        };
        (
            content_trait: $content_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $content_type_name:ident = {
                kind: Static,
                the_type: $static_type:ty
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                $(#[$attributes])*
                #[sbor(transparent)]
                pub struct $content_type_name(pub $static_type);
                impl From<$static_type> for $content_type_name {
                    fn from(value: $static_type) -> Self {
                        Self(value)
                    }
                }
                impl $content_trait<$content_type_name> for $static_type {}
            }
        };
        (
            content_trait: $content_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $content_type_name:ident = {
                kind: Instance,
                ident: $instance_ident:ident
                $(,)?
            }
        ) => {
            $(#[$attributes])*
            #[sbor(transparent)]
            pub struct $content_type_name<$instance_ident = ScryptoValue>(pub $instance_ident);
        };
        // TODO - Add support for some kind of StaticMultiVersioned type here
    }

    #[allow(unused)]
    pub(crate) use generate_content_type;

    macro_rules! generate_system_substate_type_alias {
        (SystemField, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around SystemField substates
            pub type $alias = $content;
        };
        (Field, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            pub type $alias = FieldSubstate<$content>;
        };
        (KeyValue, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            pub type $alias = KeyValueEntrySubstate<$content>;
        };
        (Index, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around Index substates
            pub type $alias = $content;
        };
        (SortedIndex, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around SortedIndex substates
            pub type $alias = $content;
        };
        ($unknown_system_substate_type:ident, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            compile_error!(concat!(
                "Unrecognized system substate type: `",
                stringify!($unknown_system_substate_type),
                "` - expected `Field`, `SystemField`, `KeyValue`, `Index` or `SortedIndex`"
            ));
        };
    }

    #[allow(unused)]
    pub(crate) use generate_system_substate_type_alias;

    macro_rules! map_collection_schema {
        (KeyValue, $aggregator:ident, $key_type:tt, $key_content_alias:ident, $value_type:tt, $value_content_alias:ident, $can_own:expr$(,)?) => {
            BlueprintCollectionSchema::KeyValueStore(BlueprintKeyValueSchema {
                key: map_type_ref!($aggregator, $key_type, $key_content_alias),
                value: map_type_ref!($aggregator, $value_type, $value_content_alias),
                can_own: $can_own,
            })
        };
        (Index, $aggregator:ident, $key_type:tt, $key_content_alias:ident, $value_type:tt, $value_content_alias:ident, $can_own:expr$(,)?) => {
            BlueprintCollectionSchema::Index(BlueprintKeyValueSchema {
                key: map_type_ref!($aggregator, $key_type, $key_content_alias),
                value: map_type_ref!($aggregator, $value_type, $value_content_alias),
                can_own: $can_own,
            })
        };
        (SortedIndex, $aggregator:ident, $key_type:tt, $key_content_alias:ident, $value_type:tt, $value_content_alias:ident, $can_own:expr$(,)?) => {
            BlueprintCollectionSchema::SortedIndex(BlueprintKeyValueSchema {
                key: map_type_ref!($aggregator, $key_type, $key_content_alias),
                value: map_type_ref!($aggregator, $value_type, $value_content_alias),
                can_own: $can_own,
            })
        };
        ($unknown_system_substate_type:ident, $aggregator:ident, $collection_key_type:tt, $collection_value_type:tt, $collection_can_own:expr$(,)?) => {
            compile_error!(concat!(
                "Unrecognized system collection substate type: `",
                stringify!($unknown_system_substate_type),
                "` - expected `KeyValue`, `Index` or `SortedIndex`"
            ));
        };
    }

    #[allow(unused)]
    pub(crate) use map_collection_schema;

    macro_rules! map_type_ref {
        (
            $aggregator:ident,
            {
                kind: StaticSingleVersioned
                $(,)?
            },
            $content_alias:ident$(,)?
        ) => {
            TypeRef::Static($aggregator.add_child_type_and_descendents::<$content_alias>())
        };
        (
            $aggregator:ident,
            {
                kind: Static,
                the_type: $static_type:ty
                $(,)?
            },
            $content_alias:ident$(,)?
        ) => {
            TypeRef::Static($aggregator.add_child_type_and_descendents::<$content_alias>())
        };
        (
            $aggregator:ident,
            {
                kind: Instance,
                ident: $instance_ident:ident
                $(,)?
            },
            $content_alias:ident$(,)?
        ) => {
            compile_error!("Instance schemas not yet supported - close though!")
        }; // TODO - Add support for some kind of StaticMultiVersioned type here
    }

    #[allow(unused)]
    pub(crate) use map_type_ref;

    macro_rules! map_entry_substate_to_kv_entry {
        (KeyValue, $entry_substate:ident) => {
            paste::paste! {
                KVEntry {
                    value: $entry_substate.value.map(|v| scrypto_encode(&v).unwrap()),
                    locked: match $entry_substate.mutability {
                        SubstateMutability::Mutable => true,
                        SubstateMutability::Immutable => false,
                    },
                }
            }
        };
        (Index, $entry_substate:ident) => {
            // This code still needs to compile, but it shouldn't be possible to execute
            panic!("Not possible to map an Index entry to a KVEntry")
        };
        (SortedIndex, $entry_substate:ident) => {
            // This code still needs to compile, but it shouldn't be possible to execute
            panic!("Not possible to map a SortedIndex entry to a KVEntry")
        };
        ($unknown_system_substate_type:ident, $entry_substate:ident) => {
            paste::paste! {
                compile_error!(concat!(
                    "Unrecognized system collection substate type: `",
                    stringify!($unknown_system_substate_type),
                    "` - expected `KeyValue`, `Index` or `SortedIndex`"
                ));
            }
        };
    }

    #[allow(unused)]
    pub(crate) use map_entry_substate_to_kv_entry;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check that the below compiles
    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct TestBlueprintRoyaltyV1;

    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct TestBlueprintMyCoolKeyValueStoreV1;

    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct TestBlueprintMyCoolIndexV1;

    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct TestBlueprintMyCoolSortedIndexV1;

    use radix_engine_interface::blueprints::package::*;

    declare_native_blueprint_state! {
        blueprint_ident: TestBlueprint,
        blueprint_snake_case: package,
        features: {},
        instance_schema_types: {},
        fields: {
            royalty:  {
                ident: Royalty,
                field_type: {
                    kind: StaticSingleVersioned,
                },
                condition: Condition::Always,
            }
        },
        collections: {
            some_key_value_store: KeyValue {
                entry_ident: MyCoolKeyValueStore,
                key_type: {
                    kind: Static,
                    the_type: BlueprintVersion,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                can_own: true,
            },
            abc: Index {
                entry_ident: MyCoolIndex,
                key_type: {
                    kind: Static,
                    the_type: BlueprintVersion,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                can_own: true,
            },
            def: SortedIndex {
                entry_ident: MyCoolSortedIndex,
                key_type: {
                    kind: Static,
                    the_type: BlueprintVersion,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                can_own: true,
            },
        }
    }
}
