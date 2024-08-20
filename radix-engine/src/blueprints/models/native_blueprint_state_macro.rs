use crate::internal_prelude::*;

/// Generates types and typed-interfaces for native blueprints, their
/// state models, features, and schemas.
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
///     {
///         kind: StaticMultiVersioned,
///         previous_versions: {
///             1 => { updates_to: 2 },
///             2 => { updates_to: 3 },
///         },
///         latest_version: 3,
///     }
/// ```
///
/// Choosing `StaticSingleVersioned`, which will create a forward-compatible enum wrapper with a single version for the content.
///
/// For Fields, it will assume the existence of a type called
/// `<BlueprintIdent><FieldIdent>V1` and will generate the following types:
/// * `<BlueprintIdent><FieldIdent>` - a type alias for the latest version (V1).
/// * `Versioned<BlueprintIdent><FieldIdent>` - the enum wrapper with a single version. This will be the content of `<BlueprintIdent><FieldIdent>FieldPayload`.
///
/// For collection values, it will assume the existence of `<BlueprintIdent><CollectionIdent>V1` and generate the following types:
/// * `<BlueprintIdent><CollectionIdent>` - a type alias for the latest version (V1).
/// * `Versioned<BlueprintIdent><CollectionIdent>` - the enum wrapper with a single version. This will be the content of `<BlueprintIdent><CollectionIdent>EntryPayload`.
///
/// Choosing `StaticMultiVersioned` will create the same types, but the type alias will be for the latest version.
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
                    pub [<Versioned $ident_core>]([<$ident_core Versions>]) => $ident_core = [<$ident_core V1>]
                );
                declare_payload_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    ----
                    $(#[$attributes])*
                    pub struct $payload_type_name([<Versioned $ident_core>]);
                );

                impl $payload_type_name
                {
                    #[doc = "Delegates to [`"[<Versioned $ident_core>]"::fully_update`]."]
                    pub fn fully_update(self) -> Self {
                        Self::of(self.content.fully_update())
                    }

                    #[doc = "Delegates to [`"[<Versioned $ident_core>]"::fully_update_and_into_latest_version`]."]
                    pub fn fully_update_and_into_latest_version(self) -> $ident_core {
                        self.content.fully_update_and_into_latest_version()
                    }

                    #[doc = "Delegates to [`"[<Versioned $ident_core>]"::from_latest_version`]."]
                    pub fn from_latest_version(latest_version: $ident_core) -> Self {
                        [<Versioned $ident_core>]::from_latest_version(latest_version).into()
                    }

                    #[doc = "Delegates to [`"[<Versioned $ident_core>]"::into_unique_version`]."]
                    pub fn into_unique_version(self) -> $ident_core {
                        self.content.into_unique_version()
                    }

                    #[doc = "Delegates to [`"[<Versioned $ident_core>]"::from_unique_version`]."]
                    pub fn from_unique_version(unique_version: $ident_core) -> Self {
                        [<Versioned $ident_core>]::from_unique_version(unique_version).into()
                    }
                }

                // Now implement the Content trait for other relevant content types that don't already have it:
                // > The internal enum: [<$ident_core Versions>]
                // > The latest (and unique) version: $ident_core = [<$ident_core V $latest_version_num>]
                impl $content_trait<$payload_type_name> for [<$ident_core Versions>] {
                    fn into_content(self) -> [<Versioned $ident_core>] {
                        [<Versioned $ident_core>]::from(self).into()
                    }
                }

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
                kind: StaticMultiVersioned,
                previous_versions: [
                    $($version_num:expr => { updates_to: $update_to_version_num:expr }),*
                    $(,)? // Optional trailing comma
                ],
                latest_version: $latest_version_num:expr
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                sbor::define_versioned!(
                    $(#[$attributes])*
                    pub [<Versioned $ident_core>]([<$ident_core Versions>]) {
                        previous_versions: [
                            $($version_num => [<$ident_core V $version_num>]: { updates_to: $update_to_version_num }),*
                        ],
                        latest_version: {
                            $latest_version_num => $ident_core = [<$ident_core V $latest_version_num>]
                        }
                    }
                );
                declare_payload_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    ----
                    $(#[$attributes])*
                    pub struct $payload_type_name([<Versioned $ident_core>]);
                );

                // NOTE: If updating here, also update StaticSingleVersioned
                impl $payload_type_name
                {
                    #[doc = "Delegates to [`"[<Versioned $ident_core>]"::fully_update`]."]
                    pub fn fully_update(self) -> Self {
                        Self::of(self.content.fully_update())
                    }

                    #[doc = "Delegates to [`"[<Versioned $ident_core>]"::fully_update_and_into_latest_version`]."]
                    pub fn fully_update_and_into_latest_version(self) -> $ident_core {
                        self.content.fully_update_and_into_latest_version()
                    }

                    #[doc = "Delegates to [`"[<Versioned $ident_core>]"::from_latest_version`]."]
                    pub fn from_latest_version(latest_version: $ident_core) -> Self {
                        [<Versioned $ident_core>]::from_latest_version(latest_version).into()
                    }
                }

                // Now implement the Content trait for other relevant content types that don't already have it:
                // > The internal enum: [<$ident_core Versions>]
                // > Each previous version: [<$ident_core V $version_num>]
                // > The latest version: $ident_core = [<$ident_core V $latest_version_num>]
                impl $content_trait<$payload_type_name> for [<$ident_core Versions>] {
                    fn into_content(self) -> [<Versioned $ident_core>] {
                        [<Versioned $ident_core>]::from(self).into()
                    }
                }

                $(
                    impl $content_trait<$payload_type_name> for [<$ident_core V $version_num>] {
                        fn into_content(self) -> [<Versioned $ident_core>] {
                            self.into()
                        }
                    }
                )*

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
                impl [<$ident_core ContentMarker>] for ScryptoRawValue<'_> {}
            }
        };
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
                kind: StaticMultiVersioned,
                previous_versions: [
                    $($version_num:expr => { updates_to: $update_to_version_num:expr }),*
                    $(,)? // Optional trailing comma
                ],
                latest_version: $latest_version_num:expr
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
        };
    }

    #[allow(unused)]
    pub(crate) use map_type_ref;

    macro_rules! map_entry_substate_to_kv_entry {
        (KeyValue, $entry_substate:ident) => {
            paste::paste! {
                KVEntry {
                    value: $entry_substate.value.map(|v| scrypto_encode(&v).unwrap()),
                    locked: match $entry_substate.lock_status {
                        LockStatus::Locked => true,
                        LockStatus::Unlocked => false,
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

    // Types and Impls required by the macro:
    #[derive(Debug, PartialEq, Eq, Sbor, Copy, Clone)]
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

    // Macro itself
    declare_native_blueprint_state! {
        blueprint_ident: TestBlueprint,
        blueprint_snake_case: test_blueprint_v1,
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
        assert_eq!(bv, payload.into_content());
    }

    #[test]
    fn validate_royalty_field_payload_mutability() {
        let mut content = VersionedTestBlueprintRoyalty::from(TestBlueprintRoyaltyV1);
        let mut payload = TestBlueprintRoyaltyFieldPayload::from(
            VersionedTestBlueprintRoyalty::from(TestBlueprintRoyaltyV1),
        );
        assert_eq!(&content, payload.as_ref());
        assert_eq!(&mut content, payload.as_mut());
        let payload = TestBlueprintRoyaltyFieldPayload::from(VersionedTestBlueprintRoyalty::from(
            TestBlueprintRoyaltyV1,
        ));
        assert_eq!(
            LockStatus::Locked,
            payload.into_locked_substate().lock_status()
        );
        let payload = TestBlueprintRoyaltyFieldPayload::from(VersionedTestBlueprintRoyalty::from(
            TestBlueprintRoyaltyV1,
        ));
        assert_eq!(
            LockStatus::Unlocked,
            payload.into_unlocked_substate().lock_status()
        );
    }

    #[test]
    fn validate_key_value_store_entry_payload_mutability() {
        fn create_payload() -> TestBlueprintMyCoolKeyValueStoreEntryPayload {
            TestBlueprintMyCoolKeyValueStoreEntryPayload::of(
                TestBlueprintMyCoolKeyValueStoreV1.into(),
            )
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

        assert!(create_payload().as_latest_version().is_some());
    }

    #[test]
    fn validate_index_entry_payload() {
        let payload = TestBlueprintMyCoolIndexEntryPayload::of(TestBlueprintMyCoolIndexV1.into());
        assert_eq!(
            payload.into_substate().into_value().into_content(),
            VersionedTestBlueprintMyCoolIndex::from(TestBlueprintMyCoolIndexV1)
        );

        let content =
            TestBlueprintMyCoolIndexVersions::V1(TestBlueprintMyCoolIndexV1).into_versioned();
        assert_eq!(
            VersionedTestBlueprintMyCoolIndex::from(TestBlueprintMyCoolIndexV1),
            content.into_substate().into_value().into_content()
        );
    }

    #[test]
    fn validate_sorted_index_entry_payload() {
        let payload =
            TestBlueprintMyCoolSortedIndexEntryPayload::of(TestBlueprintMyCoolSortedIndexV1.into());
        assert_eq!(
            payload.into_substate().into_value().into_content(),
            TestBlueprintMyCoolSortedIndexVersions::V1(TestBlueprintMyCoolSortedIndexV1)
                .into_versioned()
        );

        let content = TestBlueprintMyCoolSortedIndexVersions::V1(TestBlueprintMyCoolSortedIndexV1)
            .into_versioned();
        assert_eq!(
            TestBlueprintMyCoolSortedIndexVersions::V1(TestBlueprintMyCoolSortedIndexV1)
                .into_versioned(),
            content.into_substate().into_value().into_content()
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

    // And now imagine we have a V2 blueprint needing types:

    pub type TestBlueprintV2RoyaltyV1 = TestBlueprintRoyaltyV1;
    pub type TestBlueprintV2MyCoolKeyValueStoreV1 = TestBlueprintMyCoolKeyValueStoreV1;
    pub type TestBlueprintV2MyCoolIndexV1 = TestBlueprintMyCoolIndexV1;
    pub type TestBlueprintV2MyCoolSortedIndexV1 = TestBlueprintMyCoolSortedIndexV1;

    pub type TestBlueprintV2PartitionOffset = TestBlueprintPartitionOffset;

    #[derive(Debug, PartialEq, Eq, Sbor, Clone)]
    pub struct TestBlueprintV2RoyaltyV2 {
        my_new_value: String,
    }

    impl From<TestBlueprintV2RoyaltyV1> for TestBlueprintV2RoyaltyV2 {
        fn from(_value: TestBlueprintV2RoyaltyV1) -> Self {
            Self {
                my_new_value: "created during upgrade".to_string(),
            }
        }
    }

    impl SortedIndexKeyFullContent<TestBlueprintV2MyCoolSortedIndexKeyPayload>
        for ExampleSortedIndexKey
    {
        fn from_sort_key_and_content(sort_key: u16, content: BlueprintVersion) -> Self {
            ExampleSortedIndexKey(sort_key, content)
        }

        fn as_content(&self) -> &BlueprintVersion {
            &self.1
        }
    }

    impl SortedIndexKeyContentSource<TestBlueprintV2MyCoolSortedIndexKeyPayload>
        for ExampleSortedIndexKey
    {
        fn sort_key(&self) -> u16 {
            self.0
        }

        fn into_content(self) -> BlueprintVersion {
            self.1
        }
    }

    declare_native_blueprint_state! {
        blueprint_ident: TestBlueprintV2,
        blueprint_snake_case: test_blueprint_v2,
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
                    kind: StaticMultiVersioned,
                    previous_versions: [
                        1 => { updates_to: 2 }
                    ],
                    latest_version: 2,
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

    #[test]
    fn validate_v1_data_can_be_loaded_as_v1_or_v2() {
        // ----------
        // V1 Content
        // ----------
        // Content v1 is used by both TestBlueprint and TestBlueprintV2 - it's actually the same type
        let content_v1 = TestBlueprintRoyaltyV1;
        let v1_versioned_content_v1 = VersionedTestBlueprintRoyalty::from(content_v1.clone());
        let v1_payload_v1 = TestBlueprintRoyaltyFieldPayload::of(content_v1.clone().into());
        let v2_versioned_content_v1 = VersionedTestBlueprintV2Royalty::from(content_v1.clone());
        let v2_payload_v1 = TestBlueprintV2RoyaltyFieldPayload::of(content_v1.clone().into());

        // These should all be the same:
        let encoded_v1_versioned_content_v1 = scrypto_encode(&v1_versioned_content_v1).unwrap();
        let encoded_v1_payload_v1 = scrypto_encode(&v1_payload_v1).unwrap();
        let encoded_v2_versioned_content_v1 = scrypto_encode(&v2_versioned_content_v1).unwrap();
        let encoded_v2_payload_v1 = scrypto_encode(&v2_payload_v1).unwrap();

        assert_eq!(encoded_v1_versioned_content_v1, encoded_v1_payload_v1);
        assert_eq!(
            encoded_v1_versioned_content_v1,
            encoded_v2_versioned_content_v1
        );
        assert_eq!(encoded_v1_versioned_content_v1, encoded_v2_payload_v1);

        let v1_payload_v1_decoded_as_v2 =
            scrypto_decode::<TestBlueprintV2RoyaltyFieldPayload>(&encoded_v1_payload_v1).unwrap();
        assert_eq!(&v1_payload_v1_decoded_as_v2, &v2_payload_v1);

        // ----------
        // V2 Content
        // ----------
        // Content v2 is used by only TestBlueprintV2:
        let content_v2 = TestBlueprintV2RoyaltyV2 {
            my_new_value: "created during upgrade".to_string(),
        };
        let v2_versioned_content_v2 = VersionedTestBlueprintV2Royalty::from(content_v2.clone());
        let v2_payload_v2 = TestBlueprintV2RoyaltyFieldPayload::of(content_v2.clone().into());

        // These should both be the same:
        let encoded_v2_versioned_content_v2 = scrypto_encode(&v2_versioned_content_v2).unwrap();
        let encoded_v2_payload_v2 = scrypto_encode(&v2_payload_v2).unwrap();
        assert_eq!(encoded_v2_versioned_content_v2, encoded_v2_payload_v2);

        // And we can check that upgrading it works:
        let v2_payload_v1_updated =
            TestBlueprintV2RoyaltyFieldPayload::of(content_v1.clone().into()).fully_update();
        assert_eq!(&v2_payload_v1_updated, &v2_payload_v2);

        // And deserializing into a v2_payload works:
        let decoded_v2_payload_v2 =
            scrypto_decode::<TestBlueprintV2RoyaltyFieldPayload>(&encoded_v2_payload_v2).unwrap();
        assert_eq!(&decoded_v2_payload_v2, &v2_payload_v2);

        // But deserializing as a v1_payload (which doesn't understand v2) breaks:
        let decode_result_v1_payload_v2 =
            scrypto_decode::<TestBlueprintRoyaltyFieldPayload>(&encoded_v2_payload_v2);
        assert!(decode_result_v1_payload_v2.is_err());
    }
}
