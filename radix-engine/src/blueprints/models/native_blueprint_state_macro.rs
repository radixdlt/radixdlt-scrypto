use crate::internal_prelude::*;

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
/// * `<BlueprintIdent><CollectionIdent>KeyPayload` - a new type for the key payload (eg includes the u16 for a sorted index key)
///
/// The content of each of the above can take a number of forms. This is configured via specifying the type as one of the following.
/// Only Static is supported for keys at present. By default, you should choose StaticSingleVersioned for fields and collection values.
/// ```ignore
///     {
///         kind: StaticSingleVersioned,
///     }
///     {
///         kind: Static,
///         content_type: x,
///     },
///     {
///         kind: Generic,
///         ident: BlueprintGenericParameterIdent,
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
        $(
            generics: {
                $(
                    $generic_property_name:ident: {
                        ident: $generic_ident:ident,
                        description: $generic_description:expr
                        $(,)?
                    }
                ),*
                $(,)?
            },
        )?
        $(
            features: {
                $(
                    $feature_property_name:ident: {
                        ident: $feature_ident:ident,
                        description: $feature_description:expr,
                    }
                ),*
                $(,)?
            },
        )?
        fields: {
            $(
                $field_property_name:ident: {
                    ident: $field_ident:ident,
                    field_type: $field_type:tt
                    $(, condition: $field_condition:expr)?
                    $(, transience: $field_transience:expr)?
                    $(,)? // Optional trailing comma
                }
            ),*
            $(,)? // Optional trailing comma
        },
        collections: {
            $(
                $collection_property_name:ident: $collection_type:ident {
                    entry_ident: $collection_ident:ident,
                    $(mapped_physical_partition: $mapped_physical_partition:expr,)?
                    key_type: $collection_key_type:tt,
                    // The full_key_content is required if it's a sorted index
                    $(full_key_content: $full_key_content:tt,)?
                    value_type: $collection_value_type:tt,
                    allow_ownership: $collection_allow_ownership:expr
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
                use $crate::internal_prelude::*;
                use $crate::track::interface::*;
                use $crate::errors::*;
                use $crate::system::system::*;
                use radix_engine_interface::api::*;
                use radix_native_sdk::prelude::*;
                //--------------------------------------------------------
                // MODELS
                //--------------------------------------------------------

                // Generate models for each field
                $(
                    // Value
                    // > Set up Versioned types (if relevant). Assumes XXFieldV1 exists and then creates
                    //   - VersionedXXField
                    //   - XXField (alias for XXFieldV1)
                    // > Set up the (transparent) XXFieldPayload new type to represent the content of the field
                    // > Set up the FieldContent trait for anything which can be resolved into the field payload
                    generate_content_type!(
                        content_trait: FieldContentSource,
                        payload_trait: FieldPayload,
                        ident_core: [<$blueprint_ident $field_ident>],
                        #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
                        struct [<$blueprint_ident $field_ident FieldPayload>] = $field_type
                    );

                    // > Set up the _FieldSubstate alias for the system-wrapped substate
                    generate_system_substate_type_alias!(
                        Field,
                        type [<$blueprint_ident $field_ident FieldSubstate>] = WRAPPED [<$blueprint_ident $field_ident FieldPayload>]
                    );

                    pub enum [<$blueprint_ident $field_ident FieldDefinition>] {}

                    impl FieldDefinition for [<$blueprint_ident $field_ident FieldDefinition>] {
                        const FIELD_INDEX: u8 = [<$blueprint_ident Field>]::$field_ident.field_index();
                        type Payload = [<$blueprint_ident $field_ident FieldPayload>];
                        type Content = < [<$blueprint_ident $field_ident FieldPayload>] as FieldPayload >::Content;
                    }
                )*

                // Generate models for each collection
                $(
                    // Key
                    generate_key_type!(
                        content_trait: [<$collection_type KeyContentSource>],
                        payload_trait: [<$collection_type KeyPayload>],
                        $(full_key_content: $full_key_content,)?
                        #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor)]
                        struct [<$blueprint_ident $collection_ident KeyPayload>] = $collection_key_type
                    );
                    pub type [<$blueprint_ident $collection_ident KeyContent>] = <[<$blueprint_ident $collection_ident KeyPayload>] as [<$collection_type KeyPayload>]>::Content;

                    // Values
                    // > If relevant, set up Versioned types, which:
                    //   - Assumes [BlueprintCollection]V1 exists
                    //   - Creates Versioned[BlueprintCollection] enum
                    //   - Creates [BlueprintCollection] as a "latest" type alias for [BlueprintCollection]V1
                    // > Set up the [BlueprintCollection]EntryPayload transparent new type for the value content
                    // > Set up the [Collectiontype]EntryContent::<[BlueprintCollection]EntryPayload> trait for:
                    //   - The [BlueprintCollection] if it exists
                    //   - The Versioned[BlueprintCollection] if it exists
                    //   - The static content type, if it exists
                    generate_content_type!(
                        content_trait: [<$collection_type EntryContentSource>],
                        payload_trait: [<$collection_type EntryPayload>],
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

                //-------------------------------------
                // System - Generate schema definitions
                //-------------------------------------
                pub struct [<$blueprint_ident StateSchemaInit>];

                impl [<$blueprint_ident StateSchemaInit>] {
                    pub fn create_schema_init(
                        type_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>,
                    ) -> BlueprintStateSchemaInit {
                        let mut fields = vec![];
                        $(
                            fields.push(FieldSchema {
                                field: map_type_ref!(
                                    $blueprint_ident,
                                    type_aggregator,
                                    $field_type,
                                    [<$blueprint_ident $field_ident FieldPayload>],
                                ),
                                condition: optional_or_fallback!($({ $field_condition })?, { Condition::Always }),
                                transience: optional_or_fallback!($({ $field_transience})?, { FieldTransience::NotTransient }),
                            });
                        )*
                        let mut collections = vec![];
                        $(
                            collections.push(map_collection_schema!(
                                $collection_type,
                                $blueprint_ident,
                                type_aggregator,
                                $collection_key_type,
                                [<$blueprint_ident $collection_ident KeyContent>],
                                $collection_value_type,
                                [<$blueprint_ident $collection_ident EntryPayload>],
                                $collection_allow_ownership
                            ));
                        )*
                        BlueprintStateSchemaInit {
                            fields,
                            collections,
                        }
                    }
                }

                //--------------------------------------------------------
                // System - Fields, Collections, Features and Generics
                //--------------------------------------------------------
                if_exists!(
                    TEST: [[$($field_ident)*]],
                    // Avoid https://doc.rust-lang.org/error_codes/E0084.html if no fields exist
                    [[
                        #[repr(u8)]
                        #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
                        pub enum [<$blueprint_ident Field>] {
                            $($field_ident,)*
                        }

                        impl [<$blueprint_ident Field>] {
                            pub const fn field_index(&self) -> u8 {
                                *self as u8
                            }
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

                        impl FieldDescriptor for [<$blueprint_ident Field>] {
                            fn field_index(&self) -> FieldIndex {
                                *self as u8
                            }
                        }
                    ]],
                    [[
                        #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
                        pub enum [<$blueprint_ident Field>] {}

                        impl FieldDescriptor for [<$blueprint_ident Field>] {
                            fn field_index(&self) -> FieldIndex {
                                unreachable!("No fields exist")
                            }
                        }
                    ]],
                );

                if_exists!(
                    TEST: [[$($collection_ident)*]],
                    [[
                        #[repr(u8)]
                        #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
                        pub enum [<$blueprint_ident Collection>] {
                            $([<$collection_ident $collection_type>],)*
                        }

                        impl TryFrom<u8> for [<$blueprint_ident Collection>] {
                            type Error = ();

                            fn try_from(offset: u8) -> Result<Self, Self::Error> {
                                Self::from_repr(offset).ok_or(())
                            }
                        }

                        impl CollectionDescriptor for [<$blueprint_ident Collection>] {
                            fn collection_index(&self) -> CollectionIndex {
                                *self as u8
                            }
                        }
                    ]],
                    [[
                        #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
                        pub enum [<$blueprint_ident Collection>] {}

                        impl CollectionDescriptor for [<$blueprint_ident Collection>] {
                            fn collection_index(&self) -> CollectionIndex {
                                unreachable!("No collections exist")
                            }
                        }
                    ]]
                );

                $(
                    #[repr(u8)]
                    #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, FromRepr)]
                    pub enum [<$blueprint_ident Generic>] {
                        $($generic_ident,)*
                    }

                    impl [<$blueprint_ident Generic>] {
                        pub const fn generic_index(&self) -> u8 {
                            *self as u8
                        }
                    }
                )?

                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash)]
                pub enum [<$blueprint_ident Feature>] {
                    $($($feature_ident,)*)?
                }

                impl BlueprintFeature for [<$blueprint_ident Feature>] {
                    fn feature_name(&self) -> &'static str {
                        if_exists!(
                            TEST: [[$($($feature_ident)*)?]],
                            [[
                                match *self {
                                    $($(
                                        Self::$feature_ident => stringify!($feature_property_name),
                                    )*)?
                                }
                            ]],
                            [[
                                unreachable!("No features exist")
                            ]]
                        )
                    }
                }

                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, Default)]
                pub struct [<$blueprint_ident FeatureSet>] {
                    $($(pub [<$feature_property_name>]: bool,)*)?
                }

                impl [<$blueprint_ident FeatureSet>] {
                    pub fn all_features() -> IndexSet<String> {
                        let mut features = index_set_new();
                        $($(
                            features.insert(
                                [<$blueprint_ident Feature>]::$feature_ident.feature_name().to_string()
                            );
                        )*)?
                        features
                    }
                }

                impl HasFeatures for [<$blueprint_ident FeatureSet>] {
                    fn feature_names_str(&self) -> Vec<&'static str> {
                        let mut names = vec![];
                        $($(
                            if self.[<$feature_property_name>] {
                                names.push([<$blueprint_ident Feature>]::$feature_ident.feature_name());
                            }
                        )*)?
                        names
                    }
                }

                //------------------------------
                // Application - Typed State API
                //------------------------------

                pub struct [<$blueprint_ident StateApi>]<'a, Y: ClientApi<RuntimeError>> {
                    native_api: NativeClientApi::<'a, Y, RuntimeError>,
                }

                impl<'a, Y: ClientApi<RuntimeError>> [<$blueprint_ident StateApi>]<'a, Y> {
                    pub fn new<T: Into<NativeClientApi::<'a, Y, RuntimeError>>>(client_api: T) -> Self {
                        Self {
                            native_api: client_api.into(),
                        }
                    }

                    /// This is intended as an "escape-glass" fallback, but in future will be
                    /// removed when it's no longer needed.
                    /// 
                    /// You can get a &mut ClientApi by doing `*api.raw_api()`
                    pub fn raw_api(&self) -> RefMut<'_, &'a mut Y> {
                        self.native_api.raw_api()
                    }

                    $(
                        pub fn [<$field_property_name _field>](&self) -> FieldSource<'_, [<$blueprint_ident $field_ident FieldDefinition>], NativeClientApi::<'a, Y, RuntimeError>, RuntimeError> {
                            FieldSource::new(&self.native_api, ACTOR_STATE_SELF)
                        }
                    )*

                    // TODO: Implement for various collections
                }

                //--------------------------------
                // System - Object Initialization
                //--------------------------------

                /// Used for initializing blueprint state.
                ///
                /// Note - this doesn't support:
                /// * IndexEntries (because the underlying new_object API doesn't support them)
                ///   > these panic at create time
                #[derive(Debug, Default)]
                pub struct [<$blueprint_ident StateInit>] {
                    $(
                        pub $field_property_name: Option<[<$blueprint_ident $field_ident FieldSubstate>]>,
                    )*
                    $(
                        pub $collection_property_name: IndexMap<
                            [<$blueprint_ident $collection_ident KeyPayload>],
                            [<$blueprint_ident $collection_ident EntrySubstate>]
                        >,
                    )*
                }

                type [<$blueprint_ident FeatureChecks>] = [<$(ignore_arg!($outer_blueprint_ident) InnerObject)? FeatureChecks>]::<
                    [<$blueprint_ident FeatureSet>],
                    $([<$outer_blueprint_ident FeatureSet>],)?
                >;

                impl [<$blueprint_ident StateInit>] {
                    pub fn into_system_substates(self, feature_checks: [<$blueprint_ident FeatureChecks>]) -> Result<(IndexMap<u8, FieldValue>, IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>), RuntimeError> {
                        let mut field_values = IndexMap::default();
                        $(
                            {
                                feature_checks.assert_valid(
                                    stringify!($field_ident),
                                    &optional_or_fallback!($({ $field_condition })?, { Condition::Always }),
                                    self.$field_property_name.is_some(),
                                )?;
                                if let Some(field) = self.$field_property_name {
                                    let payload = scrypto_encode(&field.payload()).unwrap();
                                    let locked = match field.lock_status() {
                                        LockStatus::Unlocked => true,
                                        LockStatus::Locked => false,
                                    };
                                    field_values.insert(
                                        [<$blueprint_ident Field>]::$field_ident.into(),
                                        FieldValue {
                                            value: payload,
                                            locked,
                                        }
                                    );
                                }
                            }
                        )*
                        let mut all_collection_entries = IndexMap::new();
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

                    pub fn into_new_object<Y: ClientObjectApi<RuntimeError>>(
                        self,
                        api: &mut Y,
                        own_features: [<$blueprint_ident FeatureSet>],
                        $(outer_object_features: [<$outer_blueprint_ident FeatureSet>],)?
                        generic_args: GenericArgs,
                    ) -> Result<NodeId, RuntimeError> {
                        let (field_values, all_collection_entries) = self.into_system_substates(
                            [<$blueprint_ident FeatureChecks>]::ForFeatures {
                                own_features,
                                $(ignore_arg!($outer_blueprint_ident) outer_object_features,)?
                            }
                        )?;
                        api.new_object(
                            stringify!($blueprint_ident),
                            own_features.feature_names_str(),
                            generic_args,
                            field_values,
                            all_collection_entries, // TODO: Change to take other collections, not just KVEntry
                        )
                    }
                }

                //--------------------------------------------------------
                // System/DB - Node Partitions & Layout
                //--------------------------------------------------------

                /// A list of all logical (real) and physical (mapped) partitions for the
                /// $blueprint_ident blueprint.
                ///
                /// Note: In future, we could add a separate LogicalPartition enum, to
                /// not include physical partitions - however it's very hard to do in
                /// declarative macro land, because enum variants can't be
                /// macro invocations (to eg filter out the physical partition types)
                #[repr(u8)]
                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
                pub enum [<$blueprint_ident Partition>] {
                    Field,
                    $([<$collection_ident $collection_type>],)*
                }

                impl [<$blueprint_ident Partition>] {
                    pub const fn description(&self) -> PartitionDescription {
                        // NOTE: This should really be a fixed mapping - but it's hard to do with declarative macros
                        // It's a const function, so might hopefully be compiled away
                        let mut module_partition_offset = 0;
                        if (*self as u8) == (Self::Field as u8) {
                            return PartitionDescription::Logical(PartitionOffset(module_partition_offset));
                        }
                        $(
                            let current_partition_description = optional_or_fallback!(
                                $({
                                    PartitionDescription::Physical($mapped_physical_partition)
                                })?,
                                {
                                    module_partition_offset += 1;
                                    PartitionDescription::Logical(PartitionOffset(module_partition_offset))
                                }
                            );
                            if (*self as u8) == (Self::[<$collection_ident $collection_type>] as u8) {
                                return current_partition_description;
                            }
                        )*
                        panic!("Partition somehow did not match for calculating its description")
                    }

                    pub const fn as_main_partition(&self) -> PartitionNumber {
                        match self.description() {
                            PartitionDescription::Physical(partition_num) => partition_num,
                            PartitionDescription::Logical(offset) => {
                                match MAIN_BASE_PARTITION.at_offset(offset) {
                                    // This map works around unwrap/expect on Option not being const
                                    Some(x) => x,
                                    None => panic!("Offset larger than allowed value")
                                }
                            }
                        }
                    }
                }

                impl TryFrom<[<$blueprint_ident Partition>]> for PartitionOffset {
                    type Error = ();

                    fn try_from(value: [<$blueprint_ident Partition>]) -> Result<Self, Self::Error> {
                        match value.description() {
                            PartitionDescription::Logical(offset) => Ok(offset),
                            PartitionDescription::Physical(partition_num) => Err(()),
                        }
                    }
                }

                impl TryFrom<PartitionOffset> for [<$blueprint_ident Partition>] {
                    type Error = ();

                    fn try_from(offset: PartitionOffset) -> Result<Self, Self::Error> {
                        // NOTE: This should really be a fixed mapping - but it's hard to do with declarative macros
                        // Hopefully this will be compiled away because Partition::description is const
                        let description = PartitionDescription::Logical(offset);
                        if description == Self::Field.description() {
                            return Ok(Self::Field);
                        }
                        $(
                            if description == Self::[<$collection_ident $collection_type>].description() {
                                return Ok(Self::[<$collection_ident $collection_type>]);
                            }
                        )*
                        Err(())
                    }
                }

                //---------------------------------
                // Typed - Substate Keys and Values
                //---------------------------------

                if_exists!(
                    TEST: [[$($field_ident)*]],
                    [[
                         enum_filter_out_ignored!(
                            /// All the SubstateKeys for all logical partitions for the $blueprint_ident blueprint.
                            /// Does not include mapped partitions, as these substates are mapped via their canonical partition.
                            #[derive(Debug, Clone)]
                            pub enum [<$blueprint_ident TypedSubstateKey>]
                            {
                                [[
                                    Field([<$blueprint_ident Field>])
                                ]],
                                $(
                                    $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                                    [[
                                        [<$collection_ident $collection_type Entry>]([<$blueprint_ident $collection_ident KeyPayload>])
                                    ]],
                                )*
                            }
                        );
                    ]],
                    [[
                        enum_filter_out_ignored!(
                            /// All the SubstateKeys for all logical partitions for the $blueprint_ident blueprint.
                            /// Does not include mapped partitions, as these substates are mapped via their canonical partition.
                            #[derive(Debug, Clone)]
                            pub enum [<$blueprint_ident TypedSubstateKey>]
                            {
                                $(
                                    $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                                    [[
                                        [<$collection_ident $collection_type Entry>]([<$blueprint_ident $collection_ident KeyPayload>])
                                    ]],
                                )*
                            }
                        );
                    ]]
                );



                impl [<$blueprint_ident TypedSubstateKey>] {
                    pub fn for_key_at_partition_offset(partition_offset: PartitionOffset, substate_key: &SubstateKey) -> Result<Self, ()> {
                        Self::for_key_in_partition(
                            &[<$blueprint_ident PartitionOffset>]::try_from(partition_offset)?,
                            substate_key,
                        )
                    }

                    if_exists!(
                        TEST: [[$($field_ident)*]],
                        [[
                            pub fn for_key_in_partition(partition: &[<$blueprint_ident PartitionOffset>], substate_key: &SubstateKey) -> Result<Self, ()> {
                                let key = match_filter_out_ignored!(match partition {
                                    [[
                                        [<$blueprint_ident PartitionOffset>]::Field => {
                                            [<$blueprint_ident TypedSubstateKey>]::Field(
                                                [<$blueprint_ident Field>]::try_from(substate_key)?
                                            )
                                        }
                                    ]],
                                    $(
                                        $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                                        [[
                                            [<$blueprint_ident PartitionOffset>]::[<$collection_ident $collection_type>] => {
                                                [<$blueprint_ident TypedSubstateKey>]::[<$collection_ident $collection_type Entry>](
                                                    [<$blueprint_ident $collection_ident KeyPayload>]::try_from(substate_key)?,
                                                )
                                            }
                                        ]],
                                    )*
                                });
                                Ok(key)
                            }
                        ]],
                        [[
                            pub fn for_key_in_partition(partition: &[<$blueprint_ident PartitionOffset>], substate_key: &SubstateKey) -> Result<Self, ()> {
                                let key = match_filter_out_ignored!(match partition {
                                    $(
                                        $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                                        [[
                                            [<$blueprint_ident PartitionOffset>]::[<$collection_ident $collection_type>] => {
                                                [<$blueprint_ident TypedSubstateKey>]::[<$collection_ident $collection_type Entry>](
                                                    [<$blueprint_ident $collection_ident KeyPayload>]::try_from(substate_key)?,
                                                )
                                            }
                                        ]],
                                    )*
                                });
                                Ok(key)
                            }
                        ]]
                    );
                }

                #[derive(Debug)]
                pub enum [<$blueprint_ident TypedFieldSubstateValue>] {
                    $($field_ident([<$blueprint_ident $field_ident FieldSubstate>]),)*
                }

                enum_filter_out_ignored!(
                    /// All the Substate values for all logical partitions for the $blueprint_ident blueprint.
                    /// Does not include mapped partitions, as these substates are mapped via their canonical partition.
                    #[derive(Debug)]
                    pub enum [<$blueprint_ident TypedSubstateValue>]
                    {
                        [[
                            Field([<$blueprint_ident TypedFieldSubstateValue>])
                        ]],
                        $(
                            $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                            [[
                                [<$collection_ident $collection_type>]([<$blueprint_ident $collection_ident EntrySubstate>])
                            ]],
                        )*
                    }
                );

                impl [<$blueprint_ident TypedSubstateValue>] {
                    pub fn from_key_and_data(key: &[<$blueprint_ident TypedSubstateKey>], data: &[u8]) -> Result<Self, DecodeError> {
                        let substate_value = match_filter_out_ignored!(match key {
                            $(
                                [[
                                    [<$blueprint_ident TypedSubstateKey>]::Field([<$blueprint_ident Field>]::$field_ident) => {
                                        [<$blueprint_ident TypedSubstateValue>]::Field(
                                            [<$blueprint_ident TypedFieldSubstateValue>]::$field_ident(scrypto_decode(data)?)
                                        )
                                    }
                                ]],
                            )*
                            $(
                                $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                                [[
                                    [<$blueprint_ident TypedSubstateKey>]::[<$collection_ident $collection_type Entry>](_) => {
                                        [<$blueprint_ident TypedSubstateValue>]::[<$collection_ident $collection_type>](
                                            scrypto_decode(data)?
                                        )
                                    }
                                ]],
                            )*
                        });
                        Ok(substate_value)
                    }
                }

                //-------------
                // Flashing
                //-------------

                /// This method converts the state init into the node substates,
                /// at a kernel / flash level of abstraction.
                ///
                /// This can further be mapped to level 0 with a call to:
                /// `.into_database_updates::<SpreadPrefixKeyMapper>(&node_id)`
                ///
                /// We decided to have this as a separate function away from
                /// the impl of [<$blueprint_ident StateInit>], as it's conceptually
                /// a helper method at a different abstraction level.
                pub fn [<map_ $blueprint_property_name _state_into_main_partition_node_substate_flash>](
                    state_init: [<$blueprint_ident StateInit>],
                    feature_checks: [<$blueprint_ident FeatureChecks>],
                ) -> Result<NodeSubstates, RuntimeError> {
                    // PartitionNumber => SubstateKey => IndexedScryptoValue
                    let mut partitions: NodeSubstates = BTreeMap::new();
                    let (mut field_values, mut kv_entries) = state_init.into_system_substates(feature_checks)?;

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
                        let collection_kv_entries = kv_entries.swap_remove(&collection_index).unwrap();
                        let collection_partition = [<$blueprint_ident Partition>]::[<$collection_ident $collection_type>];
                        let collection_partition_substates = collection_kv_entries
                            .into_iter()
                            .filter_map(|(key_bytes, kv_entry)| {
                                let substate = match kv_entry {
                                    KVEntry { value: Some(value_bytes), locked: false } => {
                                        KeyValueEntrySubstate::unlocked_entry(
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
            payload_trait: $payload_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: StaticSingleVersioned
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                sbor::define_single_versioned!(
                    $(#[$attributes])*
                    pub enum [<Versioned $ident_core>] => $ident_core = [<$ident_core V1>]
                );
                declare_payload_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    ----
                    $(#[$attributes])*
                    pub struct $payload_type_name([<Versioned $ident_core>]);
                );

                impl HasLatestVersion for $payload_type_name
                {
                    type Latest = <[<Versioned $ident_core>] as HasLatestVersion>::Latest;
                    
                    fn is_latest(&self) -> bool {
                        self.as_ref().is_latest()
                    }

                    fn into_latest(self) -> Self::Latest {
                        self.into_content().into_latest()
                    }

                    fn into_latest_mut(&mut self) -> &mut Self::Latest {
                        self.as_mut().into_latest_mut()
                    }

                    fn from_latest(latest: Self::Latest) -> Self {
                        Self {
                            content: <[<Versioned $ident_core>] as HasLatestVersion>::from_latest(latest),
                        }
                    }

                    fn as_latest_ref(&self) -> Option<&Self::Latest> {
                        self.as_ref().as_latest_ref()
                    }

                    fn as_latest_mut(&mut self) -> Option<&mut Self::Latest> {
                        self.as_mut().as_latest_mut()
                    }
                }

                // Now implement other relevant content traits, for:
                // > The "latest" type: $ident_core
                impl $content_trait<$payload_type_name> for $ident_core {
                    fn into_content(self) -> [<Versioned $ident_core>] {
                        self.into()
                    }
                }
            }
        };
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: Static,
                content_type: $static_type:ty
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                declare_payload_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    ----
                    $(#[$attributes])*
                    pub struct $payload_type_name($static_type);
                );
            }
        };
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: Generic,
                ident: $generic_ident:ident
                $(,)?
            }
        ) => {
            paste::paste! {
                declare_payload_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    ----
                    $(#[$attributes])*
                    pub struct $payload_type_name<$generic_ident: [<$ident_core ContentMarker>] = ScryptoValue>($generic_ident);
                );
                // We choose to create an explicit marker trait, as an alternative to a blanket impl
                // over ScryptoEncode + ScryptoDecode. Any explicit types can implement this trait.
                // This avoids every type getting implementations for every such generic type,
                // which would require disambiguation everywhere `to_substate()` is used.
                // Anyone needing a type implementing content can use the payload type itself
                pub trait [<$ident_core ContentMarker>]: ScryptoEncode + ScryptoDecode {}
                impl [<$ident_core ContentMarker>] for ScryptoValue {}
                impl [<$ident_core ContentMarker>] for RawScryptoValue<'_> {}
            }
        };
        // TODO - Add support for some kind of StaticMultiVersioned type here
    }

    #[allow(unused)]
    pub(crate) use generate_content_type;

    macro_rules! generate_key_type {
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            $(full_key_content: $full_key_content:tt,)?
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: StaticSingleVersioned
                $(,)?
            }$(,)?
        ) => {
            compile_error!(
                "A StaticSingleVersioned key is not supported, because keys cannot be lazily updated, because they need to be static"
            );
        };
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            $(full_key_content: $full_key_content:tt,)?
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: Static,
                content_type: $static_type:ty
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                declare_key_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    $(full_key_content: $full_key_content,)?
                    ----
                    $(#[$attributes])*
                    pub struct $payload_type_name($static_type);
                );
            }
        };
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            $(full_key_content: $full_key_content:tt,)?
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: Generic,
                ident: $generic_ident:ident
                $(,)?
            }
        ) => {
            paste::paste! {
                compile_error!(
                    "A Generic key is not currently supported by these macros"
                );
            }
        };
    }

    #[allow(unused)]
    pub(crate) use generate_key_type;

    macro_rules! generate_system_substate_type_alias {
        (SystemField, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around SystemField substates
            pub type $alias = $content;
        };
        (Field, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            pub type $alias = $crate::system::system_substates::FieldSubstate<$content>;
        };
        (KeyValue, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            pub type $alias = KeyValueEntrySubstate<$content>;
        };
        (Index, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around Index substates
            pub type $alias = IndexEntrySubstate<$content>;
        };
        (SortedIndex, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around SortedIndex substates
            pub type $alias = SortedIndexEntrySubstate<$content>;
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
        (KeyValue, $blueprint_ident:ident, $aggregator:ident, $key_type:tt, $key_payload_alias:ident, $value_type:tt, $value_payload_alias:ident, $allow_ownership:expr$(,)?) => {
            BlueprintCollectionSchema::KeyValueStore(BlueprintKeyValueSchema {
                key: map_type_ref!($blueprint_ident, $aggregator, $key_type, $key_payload_alias),
                value: map_type_ref!(
                    $blueprint_ident,
                    $aggregator,
                    $value_type,
                    $value_payload_alias
                ),
                allow_ownership: $allow_ownership,
            })
        };
        (Index, $blueprint_ident:ident, $aggregator:ident, $key_type:tt, $key_payload_alias:ident, $value_type:tt, $value_payload_alias:ident, $allow_ownership:expr$(,)?) => {
            BlueprintCollectionSchema::Index(BlueprintKeyValueSchema {
                key: map_type_ref!($blueprint_ident, $aggregator, $key_type, $key_payload_alias),
                value: map_type_ref!(
                    $blueprint_ident,
                    $aggregator,
                    $value_type,
                    $value_payload_alias
                ),
                allow_ownership: $allow_ownership,
            })
        };
        (SortedIndex, $blueprint_ident:ident, $aggregator:ident, $key_type:tt, $key_payload_alias:ident, $value_type:tt, $value_payload_alias:ident, $allow_ownership:expr$(,)?) => {
            BlueprintCollectionSchema::SortedIndex(BlueprintKeyValueSchema {
                key: map_type_ref!($blueprint_ident, $aggregator, $key_type, $key_payload_alias),
                value: map_type_ref!(
                    $blueprint_ident,
                    $aggregator,
                    $value_type,
                    $value_payload_alias
                ),
                allow_ownership: $allow_ownership,
            })
        };
        ($unknown_system_substate_type:ident, $blueprint_ident:ident, $aggregator:ident, $key_type:tt, $key_payload_alias:ident, $value_type:tt, $value_payload_alias:ident, $allow_ownership:expr$(,)?) => {
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
            $blueprint_ident:ident,
            $aggregator:ident,
            {
                kind: StaticSingleVersioned
                $(,)?
            },
            $payload_alias:ident$(,)?
        ) => {
            TypeRef::Static($aggregator.add_child_type_and_descendents::<$payload_alias>())
        };
        (
            $blueprint_ident:ident,
            $aggregator:ident,
            {
                kind: Static,
                content_type: $static_type:ty
                $(,)?
            },
            $payload_alias:ident$(,)?
        ) => {
            TypeRef::Static($aggregator.add_child_type_and_descendents::<$payload_alias>())
        };
        (
            $blueprint_ident:ident,
            $aggregator:ident,
            {
                kind: Generic,
                ident: $generic_ident:ident
                $(,)?
            },
            $payload_alias:ident$(,)?
        ) => {
            paste::paste! {
                TypeRef::Generic([<$blueprint_ident Generic>]::$generic_ident.generic_index())
            }
        }; // TODO - Add support for some kind of StaticMultiVersioned type here
    }

    #[allow(unused)]
    pub(crate) use map_type_ref;

    macro_rules! map_entry_substate_to_kv_entry {
        (KeyValue, $entry_substate:ident) => {
            paste::paste! {
                KVEntry {
                    locked: match $entry_substate.lock_status() {
                        LockStatus::Locked => true,
                        LockStatus::Unlocked => false,
                    },
                    value: $entry_substate.into_value().map(|v| scrypto_encode(&v).unwrap()),
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

    macro_rules! optional_or_fallback {
        ($value:tt, $fallback:tt$(,)?) => {
            $value
        };
        (, $fallback:tt$(,)?) => {
            $fallback
        };
    }

    #[allow(unused)]
    pub(crate) use optional_or_fallback;

    macro_rules! enum_filter_out_ignored {
        (
            $(#[$attributes:meta])*
            pub enum $enum_name:ident
            {$(
                $(|IGNORE_ENTRY| { $($misc:tt)* } [[ $($ignored:tt)* ]])?
                $([[ $($present:tt)* ]])?
                ,
            )*}
        ) => {
            $(#[$attributes])*
            pub enum $enum_name
            {$(
                $($($present)*,)?
            )*}
        };
    }
    #[allow(unused)]
    pub(crate) use enum_filter_out_ignored;

    macro_rules! match_filter_out_ignored {
        (
            match $value_to_match:ident {$(
                $(|IGNORE_ENTRY| { $($misc:tt)*} [[ $($ignored:tt)* ]])?
                $([[ $($present:tt)* ]])?
                ,
            )*}
        ) => {
            match $value_to_match {$(
                $($($present)*)?
            )*}
        };
    }
    #[allow(unused)]
    pub(crate) use match_filter_out_ignored;

    macro_rules! if_exists {
        (
            TEST: [[]],
            [[ $($present:tt)* ]],
            [[ $($not_present:tt)* ]]$(,)?
        ) => {
            $($not_present)*
        };
        (
            TEST: [[ $($exists:tt)* ]],
            [[ $($present:tt)* ]],
            [[ $($not_present:tt)* ]]$(,)?
        ) => {
            $($present)*
        };
    }
    #[allow(unused)]
    pub(crate) use if_exists;
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

    #[allow(dead_code)]
    pub enum TestBlueprintPartitionOffset {
        Field,
        MyCoolKeyValueStoreKeyValue,
        MyCoolIndexIndex,
        MyCoolSortedIndexSortedIndex,
    }

    impl TryFrom<PartitionOffset> for TestBlueprintPartitionOffset {
        type Error = ();

        fn try_from(_value: PartitionOffset) -> Result<Self, Self::Error> {
            Err(())
        }
    }

    declare_native_blueprint_state! {
        blueprint_ident: TestBlueprint,
        blueprint_snake_case: package,
        generics: {
            abc: {
                ident: Abc,
                description: "Some generic parameter called Abc",
            }
        },
        features: {
            some_feature: {
                ident: Feature,
                description: "Some feature",
            }
        },
        fields: {
            royalty:  {
                ident: Royalty,
                field_type: {
                    kind: StaticSingleVersioned,
                },
                condition: Condition::Always,
            },
            some_generic_field:  {
                ident: GenericField,
                field_type: {
                    kind: Generic,
                    ident: Abc,
                },
            }
        },
        collections: {
            some_key_value_store: KeyValue {
                entry_ident: MyCoolKeyValueStore,
                key_type: {
                    kind: Static,
                    content_type: BlueprintVersion,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                allow_ownership: true,
            },
            abc: Index {
                entry_ident: MyCoolIndex,
                key_type: {
                    kind: Static,
                    content_type: BlueprintVersion,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                allow_ownership: true,
            },
            def: SortedIndex {
                entry_ident: MyCoolSortedIndex,
                key_type: {
                    kind: Static,
                    content_type: BlueprintVersion,
                },
                full_key_content: {
                    full_content_type: ExampleSortedIndexKey,
                    sort_prefix_property_name: sort_prefix,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                allow_ownership: true,
            },
        }
    }

    pub struct ExampleSortedIndexKey(u16, BlueprintVersion);

    impl SortedIndexKeyFullContent<TestBlueprintMyCoolSortedIndexKeyPayload> for ExampleSortedIndexKey {
        fn from_sort_key_and_content(sort_key: u16, content: BlueprintVersion) -> Self {
            ExampleSortedIndexKey(sort_key, content)
        }

        fn as_content(&self) -> &BlueprintVersion {
            &self.1
        }
    }

    impl SortedIndexKeyContentSource<TestBlueprintMyCoolSortedIndexKeyPayload>
        for ExampleSortedIndexKey
    {
        fn sort_key(&self) -> u16 {
            self.0
        }

        fn into_content(self) -> BlueprintVersion {
            self.1
        }
    }

    #[test]
    fn validate_declare_sorted_index_key_new_type_macro() {
        let mut bv = BlueprintVersion::default();
        let mut idx_key = TestBlueprintMyCoolSortedIndexKeyPayload::new(1, bv);

        assert_eq!(&bv, idx_key.as_ref());
        assert_eq!(&mut bv, idx_key.as_mut());
        assert_eq!((1, &bv), idx_key.as_sort_key_and_content());
        assert_eq!((1, bv), idx_key.into_sort_key_and_content());
    }

    #[test]
    fn validate_declare_index_key_new_type_macro() {
        let mut bv = BlueprintVersion::default();
        let mut payload = TestBlueprintMyCoolIndexKeyPayload::from(bv);

        assert_eq!(&bv, payload.as_ref());
        assert_eq!(&mut bv, payload.as_mut());
        assert_eq!(
            bv,
            IndexKeyContentSource::into_content(payload.into_content())
        );
    }

    #[test]
    fn validate_royalty_field_payload_mutability() {
        let mut content = VersionedTestBlueprintRoyalty::V1(TestBlueprintRoyaltyV1);
        let mut payload = TestBlueprintRoyaltyFieldPayload {
            content: VersionedTestBlueprintRoyalty::V1(TestBlueprintRoyaltyV1),
        };
        assert_eq!(&content, payload.as_ref());
        assert_eq!(&mut content, payload.as_mut());
        assert_eq!(
            &LockStatus::Locked,
            payload.into_locked_substate().lock_status()
        );

        assert_eq!(
            &LockStatus::Locked,
            TestBlueprintRoyaltyV1.into_locked_substate().lock_status()
        );
        assert_eq!(
            &LockStatus::Unlocked,
            TestBlueprintRoyaltyV1
                .into_unlocked_substate()
                .lock_status()
        );
    }

    #[test]
    fn validate_key_value_store_entry_payload_mutability() {
        fn create_payload() -> TestBlueprintMyCoolKeyValueStoreEntryPayload {
            TestBlueprintMyCoolKeyValueStoreEntryPayload {
                content: VersionedTestBlueprintMyCoolKeyValueStore::V1(
                    TestBlueprintMyCoolKeyValueStoreV1,
                ),
            }
        }

        assert_eq!(
            LockStatus::Locked,
            create_payload().into_locked_substate().lock_status()
        );
        assert_eq!(
            LockStatus::Unlocked,
            create_payload().into_unlocked_substate().lock_status()
        );

        assert_eq!(
            LockStatus::Locked,
            create_payload()
                .into_content()
                .into_locked_substate()
                .lock_status()
        );
        assert_eq!(
            LockStatus::Unlocked,
            create_payload()
                .into_content()
                .into_unlocked_substate()
                .lock_status()
        );

        assert!(create_payload().as_latest_ref().is_some());
    }

    #[test]
    fn validate_index_entry_payload() {
        let payload = TestBlueprintMyCoolIndexEntryPayload {
            content: VersionedTestBlueprintMyCoolIndex::V1(TestBlueprintMyCoolIndexV1),
        };
        assert_eq!(
            payload.into_substate().value().content,
            VersionedTestBlueprintMyCoolIndex::V1(TestBlueprintMyCoolIndexV1)
        );

        let content = VersionedTestBlueprintMyCoolIndex::V1(TestBlueprintMyCoolIndexV1);
        assert_eq!(
            VersionedTestBlueprintMyCoolIndex::V1(TestBlueprintMyCoolIndexV1),
            content.into_substate().value().content
        );
    }

    #[test]
    fn validate_sorted_index_entry_payload() {
        let payload = TestBlueprintMyCoolSortedIndexEntryPayload {
            content: VersionedTestBlueprintMyCoolSortedIndex::V1(TestBlueprintMyCoolSortedIndexV1),
        };
        assert_eq!(
            payload.into_substate().value().content,
            VersionedTestBlueprintMyCoolSortedIndex::V1(TestBlueprintMyCoolSortedIndexV1)
        );

        let content = VersionedTestBlueprintMyCoolSortedIndex::V1(TestBlueprintMyCoolSortedIndexV1);
        assert_eq!(
            VersionedTestBlueprintMyCoolSortedIndex::V1(TestBlueprintMyCoolSortedIndexV1),
            content.into_substate().value().content
        );
    }

    #[test]
    fn test_blueprint_field_try_from() {
        assert!(TestBlueprintField::try_from(&SubstateKey::Field(0)).is_ok());
        assert!(TestBlueprintField::try_from(&SubstateKey::Map(Vec::new())).is_err());
    }

    #[test]
    fn validate_blueprint_field_index() {
        let field = TestBlueprintField::Royalty;
        assert_eq!(0, FieldDescriptor::field_index(&field));

        let field = TestBlueprintField::GenericField;
        assert_eq!(1, FieldDescriptor::field_index(&field));
    }

    #[test]
    fn test_substate_key_partition() {
        assert!(TestBlueprintTypedSubstateKey::for_key_at_partition_offset(
            PartitionOffset(0),
            &SubstateKey::Field(0)
        )
        .is_err());

        assert!(TestBlueprintTypedSubstateKey::for_key_in_partition(
            &TestBlueprintPartitionOffset::Field,
            &SubstateKey::Field(0)
        )
        .is_ok());

        assert!(TestBlueprintTypedSubstateKey::for_key_in_partition(
            &TestBlueprintPartitionOffset::MyCoolIndexIndex,
            &SubstateKey::Map(vec![92, 0])
        )
        .is_err());
    }
}
