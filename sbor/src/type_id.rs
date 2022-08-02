/**
 * NOTE: In the below, + means byte concatenation
 * 
 * Encoding modes:
 * 
 * > "with_type" means that type data is included so that data can be decoded without a codec/schema
 * > "without_type" means that type data known statically by the codec is not output.
 * 
 * A type encoding is in two parts:
 * 
 * > STATIC_TYPE_DATA
 *   Type data statically computable from the type being encoded.
 *   This type data is fixed for the type and can be lifted up to its ancestor types where
 *   this would result in improved performance / deduplication (eg parent collection types).
 *   This data can be exluded in the "without_type" encoding.
 * > VALUE
 *   The encoded value itself. This part can differ depending on the runtime value being encoded.
 *   This value encoding can itself output STATIC_TYPE_DATA of child types, for runtime resolution
 *   when type information can't be  "without_type" encoding mode.
 * 
 * STATIC_TYPE_DATA is TYPE_ID + EXTRA_TYPE_DATA. Each TYPE_ID below includes a comment explaining
 * how it encodes its EXTRA_TYPE_DATA (ETD), and VALUE.
 */

// Primitive types: General
pub const TYPE_UNIT: u8 = 0x00;
pub const TYPE_BOOL: u8 = 0x01;

// Primitive types: Signed Integers
pub const TYPE_I8: u8 = 0x13;
pub const TYPE_I16: u8 = 0x14;
pub const TYPE_I32: u8 = 0x15;
pub const TYPE_I64: u8 = 0x16;
pub const TYPE_I128: u8 = 0x17;

// Primitive types: Unsigned Integers
pub const TYPE_U8: u8 = 0x23;
pub const TYPE_U16: u8 = 0x24;
pub const TYPE_U32: u8 = 0x25;
pub const TYPE_U64: u8 = 0x26;
pub const TYPE_U128: u8 = 0x27;

// Primitive types: Other
pub const TYPE_STRING_KNOWN_SIZE: u8 = 0x30; // ETD: (length), value: (concat of UTF8 chars)
pub const TYPE_STRING_VAR_SIZE: u8 = 0x30; // ETD: (), value: (size + concat of UTF8 chars)

// Collection types: Array
pub const TYPE_ARRAY_KNOWN_SIZE: u8 = 0x40; // ETD: (size + item_type_data), value: (concat of each value)
pub const TYPE_ARRAY_VAR_SIZE: u8 = 0x41; // ETD: (item_type_data), value: (size + concat of each value)

// Collection types: Set
pub const TYPE_SET_KNOWN_SIZE: u8 = 0x42; // ETD: (size + item_type_data), value: (concat of each ordered value)
pub const TYPE_SET_VAR_SIZE: u8 = 0x43; // ETD: (item_type_data), value: (size + concat of each value)

// Collection types: Map
pub const TYPE_MAP_KNOWN_SIZE: u8 = 0x44; // ETD: (size + key_type_data + value_type_data), value: (concat of each (key + value))
pub const TYPE_MAP_VAR_SIZE: u8 = 0x45; // ETD: (key_type_data + value_type_data), value: (size + concat of each (key + value))

// >> Sum types: Generic
pub const TYPE_SUM_TYPE_GENERIC: u8 = 0x50; // ETD: (discriminator_type_data), value: (discriminator_value + value_type_data + value)

// >> Sum types: Explicit
pub const TYPE_OPTION: u8 = 0x52; // ETD: () - discriminator type is u8 (specifically OPTION_TYPE_NONE or OPTION_TYPE_SOME), value as above
pub const TYPE_RESULT: u8 = 0x53; // ETD: () - discriminator type is u8 (specifically RESULT_TYPE_OK or RESULT_TYPE_ERR), value as above
pub const TYPE_SUM_TYPE_STRING_KEY: u8 = 0x54; // ETD: () - discriminator_type_data = string, value as above

// >> Product types: Generic
pub const TYPE_PRODUCT_TYPE_FIXED: u8 = 0x60; // We know exactly the types in the product type. ETD: (size + concat of type_data[i]); Value: (concat of value[i])
pub const TYPE_PRODUCT_TYPE_KNOWN_SIZE: u8 = 0x60; // We know the size of the product type. ETD: (size); Value: (concat of (type_data[i] + value[i]))
pub const TYPE_PRODUCT_TYPE_VAR_SIZE: u8 = 0x61;  // Any product type. ETD: (); Value: size + (loop over i: type_data[i] + value[i])

// Custom types start from 0x80 and values are encoded as `len + data`
pub const TYPE_CUSTOM_START: u8 = 0x80;

//------

// Explicit subtypes of sum-types
pub const OPTION_TYPE_NONE: u8 = 0x00;
pub const OPTION_TYPE_SOME: u8 = 0x01;
pub const RESULT_TYPE_OK: u8 = 0x00;
pub const RESULT_TYPE_ERR: u8 = 0x01;

//------
