use crate::address::Bech32Encoder;
use crate::api::types::*;
use crate::data::*;
use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use utils::ContextualDisplay;

#[derive(Clone, Copy, Debug)]
pub struct ValueFormattingContext<'a> {
    bech32_encoder: Option<&'a Bech32Encoder>,
    bucket_names: Option<&'a HashMap<BucketId, String>>,
    proof_names: Option<&'a HashMap<ProofId, String>>,
}

impl<'a> ValueFormattingContext<'a> {
    pub fn no_context() -> Self {
        Self {
            bech32_encoder: None,
            bucket_names: None,
            proof_names: None,
        }
    }

    pub fn no_manifest_context(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            bucket_names: None,
            proof_names: None,
        }
    }

    pub fn with_manifest_context(
        bech32_encoder: Option<&'a Bech32Encoder>,
        bucket_names: &'a HashMap<BucketId, String>,
        proof_names: &'a HashMap<ProofId, String>,
    ) -> Self {
        Self {
            bech32_encoder,
            bucket_names: Some(bucket_names),
            proof_names: Some(proof_names),
        }
    }

    pub fn get_bucket_name(&self, bucket_id: &BucketId) -> Option<&str> {
        self.bucket_names
            .and_then(|names| names.get(bucket_id).map(|s| s.as_str()))
    }

    pub fn get_proof_name(&self, proof_id: &ProofId) -> Option<&str> {
        self.proof_names
            .and_then(|names| names.get(proof_id).map(|s| s.as_str()))
    }
}

impl<'a> Into<ValueFormattingContext<'a>> for &'a Bech32Encoder {
    fn into(self) -> ValueFormattingContext<'a> {
        ValueFormattingContext::no_manifest_context(Some(self))
    }
}

impl<'a> Into<ValueFormattingContext<'a>> for Option<&'a Bech32Encoder> {
    fn into(self) -> ValueFormattingContext<'a> {
        ValueFormattingContext::no_manifest_context(self)
    }
}

pub fn format_scrypto_value<F: fmt::Write>(
    f: &mut F,
    value: &ScryptoValue,
    context: &ValueFormattingContext,
) -> fmt::Result {
    match value {
        // primitive types
        SborValue::Unit => write!(f, "()")?,
        SborValue::Bool { value } => write!(f, "{}", value)?,
        SborValue::I8 { value } => write!(f, "{}i8", value)?,
        SborValue::I16 { value } => write!(f, "{}i16", value)?,
        SborValue::I32 { value } => write!(f, "{}i32", value)?,
        SborValue::I64 { value } => write!(f, "{}i64", value)?,
        SborValue::I128 { value } => write!(f, "{}i128", value)?,
        SborValue::U8 { value } => write!(f, "{}u8", value)?,
        SborValue::U16 { value } => write!(f, "{}u16", value)?,
        SborValue::U32 { value } => write!(f, "{}u32", value)?,
        SborValue::U64 { value } => write!(f, "{}u64", value)?,
        SborValue::U128 { value } => write!(f, "{}u128", value)?,
        SborValue::String { value } => write!(f, "\"{}\"", value)?,
        SborValue::Tuple { fields } => {
            f.write_str("Tuple(")?;
            format_elements(f, fields, context)?;
            f.write_str(")")?;
        }
        SborValue::Enum {
            discriminator,
            fields,
        } => {
            f.write_str("Enum(\"")?;
            f.write_str(discriminator)?;
            f.write_str("\"")?;
            if !fields.is_empty() {
                f.write_str(", ")?;
                format_elements(f, fields, context)?;
            }
            f.write_str(")")?;
        }
        SborValue::Array {
            element_type_id,
            elements,
        } => {
            f.write_str("Array<")?;
            format_type_id(f, element_type_id)?;
            f.write_str(">(")?;
            format_elements(f, elements, context)?;
            f.write_str(")")?;
        }
        // custom types
        SborValue::Custom { value } => {
            format_custom_value(f, value, context)?;
        }
    };
    Ok(())
}

pub fn format_type_id<F: fmt::Write>(f: &mut F, type_id: &ScryptoSborTypeId) -> fmt::Result {
    match type_id {
        SborTypeId::Unit => f.write_str("Unit"),
        SborTypeId::Bool => f.write_str("Bool"),
        SborTypeId::I8 => f.write_str("I8"),
        SborTypeId::I16 => f.write_str("I16"),
        SborTypeId::I32 => f.write_str("I32"),
        SborTypeId::I64 => f.write_str("I64"),
        SborTypeId::I128 => f.write_str("I128"),
        SborTypeId::U8 => f.write_str("U8"),
        SborTypeId::U16 => f.write_str("U16"),
        SborTypeId::U32 => f.write_str("U32"),
        SborTypeId::U64 => f.write_str("U64"),
        SborTypeId::U128 => f.write_str("U128"),
        SborTypeId::String => f.write_str("String"),
        SborTypeId::Enum => f.write_str("Enum"),
        SborTypeId::Array => f.write_str("Array"),
        SborTypeId::Tuple => f.write_str("Tuple"),
        SborTypeId::Custom(type_id) => match type_id {
            ScryptoCustomTypeId::PackageAddress => f.write_str("PackageAddress"),
            ScryptoCustomTypeId::ComponentAddress => f.write_str("ComponentAddress"),
            ScryptoCustomTypeId::ResourceAddress => f.write_str("ResourceAddress"),
            ScryptoCustomTypeId::SystemAddress => f.write_str("SystemAddress"),
            ScryptoCustomTypeId::Component => f.write_str("Component"),
            ScryptoCustomTypeId::KeyValueStore => f.write_str("KeyValueStore"),
            ScryptoCustomTypeId::Bucket => f.write_str("Bucket"),
            ScryptoCustomTypeId::Proof => f.write_str("Proof"),
            ScryptoCustomTypeId::Vault => f.write_str("Vault"),
            ScryptoCustomTypeId::Expression => f.write_str("Expression"),
            ScryptoCustomTypeId::Blob => f.write_str("Blob"),
            ScryptoCustomTypeId::NonFungibleAddress => f.write_str("NonFungibleAddress"),
            ScryptoCustomTypeId::Hash => f.write_str("Hash"),
            ScryptoCustomTypeId::EcdsaSecp256k1PublicKey => f.write_str("EcdsaSecp256k1PublicKey"),
            ScryptoCustomTypeId::EcdsaSecp256k1Signature => f.write_str("EcdsaSecp256k1Signature"),
            ScryptoCustomTypeId::EddsaEd25519PublicKey => f.write_str("EddsaEd25519PublicKey"),
            ScryptoCustomTypeId::EddsaEd25519Signature => f.write_str("EddsaEd25519Signature"),
            ScryptoCustomTypeId::Decimal => f.write_str("Decimal"),
            ScryptoCustomTypeId::PreciseDecimal => f.write_str("PreciseDecimal"),
            ScryptoCustomTypeId::NonFungibleId => f.write_str("NonFungibleId"),
        },
    }
}

pub fn format_elements<F: fmt::Write>(
    f: &mut F,
    values: &[ScryptoValue],
    context: &ValueFormattingContext,
) -> fmt::Result {
    for (i, x) in values.iter().enumerate() {
        if i != 0 {
            f.write_str(", ")?;
        }
        format_scrypto_value(f, x, context)?;
    }
    Ok(())
}

pub fn format_custom_value<F: fmt::Write>(
    f: &mut F,
    value: &ScryptoCustomValue,
    context: &ValueFormattingContext,
) -> fmt::Result {
    match value {
        // Global address types
        ScryptoCustomValue::PackageAddress(value) => {
            f.write_str("PackageAddress(\"")?;
            value
                .format(f, context.bech32_encoder)
                .expect("Failed to format address");
            f.write_str("\")")?;
        }
        ScryptoCustomValue::ComponentAddress(value) => {
            f.write_str("ComponentAddress(\"")?;
            value
                .format(f, context.bech32_encoder)
                .expect("Failed to format address");
            f.write_str("\")")?;
        }
        ScryptoCustomValue::ResourceAddress(value) => {
            f.write_str("ResourceAddress(\"")?;
            value
                .format(f, context.bech32_encoder)
                .expect("Failed to format address");
            f.write_str("\")")?;
        }
        ScryptoCustomValue::SystemAddress(value) => {
            f.write_str("SystemAddress(\"")?;
            value
                .format(f, context.bech32_encoder)
                .expect("Failed to format address");
            f.write_str("\")")?;
        }
        // RE node types
        ScryptoCustomValue::Component(value) => {
            write!(f, "Component(\"{}\")", hex::encode(value))?;
        }
        ScryptoCustomValue::KeyValueStore(value) => {
            write!(f, "KeyValueStore(\"{}\")", hex::encode(value))?;
        }
        ScryptoCustomValue::Bucket(value) => {
            if let Some(name) = context.get_bucket_name(&value) {
                write!(f, "Bucket(\"{}\")", name)?;
            } else {
                write!(f, "Bucket({}u32)", value)?;
            }
        }
        ScryptoCustomValue::Proof(value) => {
            if let Some(name) = context.get_proof_name(&value) {
                write!(f, "Proof(\"{}\")", name)?;
            } else {
                write!(f, "Proof({}u32)", value)?;
            }
        }
        ScryptoCustomValue::Vault(value) => {
            write!(f, "Vault(\"{}\")", hex::encode(value))?;
        }
        // Other interpreted types
        ScryptoCustomValue::Expression(value) => {
            write!(f, "Expression(\"{}\")", value)?;
        }
        ScryptoCustomValue::Blob(value) => {
            write!(f, "Blob(\"{}\")", value)?;
        }
        ScryptoCustomValue::NonFungibleAddress(value) => {
            write!(f, "NonFungibleAddress(\"{}\")", value)?;
        }
        // Uninterpreted
        ScryptoCustomValue::Hash(value) => {
            write!(f, "Hash(\"{}\")", value)?;
        }
        ScryptoCustomValue::EcdsaSecp256k1PublicKey(value) => {
            write!(f, "EcdsaSecp256k1PublicKey(\"{}\")", value)?;
        }
        ScryptoCustomValue::EcdsaSecp256k1Signature(value) => {
            write!(f, "EcdsaSecp256k1Signature(\"{}\")", value)?;
        }
        ScryptoCustomValue::EddsaEd25519PublicKey(value) => {
            write!(f, "EddsaEd25519PublicKey(\"{}\")", value)?;
        }
        ScryptoCustomValue::EddsaEd25519Signature(value) => {
            write!(f, "EddsaEd25519Signature(\"{}\")", value)?;
        }
        ScryptoCustomValue::Decimal(value) => {
            write!(f, "Decimal(\"{}\")", value)?;
        }
        ScryptoCustomValue::PreciseDecimal(value) => {
            write!(f, "PreciseDecimal(\"{}\")", value)?;
        }
        ScryptoCustomValue::NonFungibleId(value) => {
            write!(f, "NonFungibleId(\"{}\")", value)?;
        }
    }
    Ok(())
}
