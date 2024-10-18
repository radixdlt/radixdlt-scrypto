use crate::data::*;
use crate::internal_prelude::*;
use crate::validation::*;
use radix_common::address::AddressBech32Encoder;
use radix_common::data::manifest::model::*;
use radix_common::data::manifest::*;
use radix_common::network::NetworkDefinition;
use sbor::rust::prelude::*;
use sbor::*;

#[derive(Debug, Clone)]
pub enum DecompileError {
    InvalidArguments,
    EncodeError(EncodeError),
    DecodeError(DecodeError),
    FormattingError(fmt::Error),
    ValueConversionError(RustToManifestValueError),
}

impl From<EncodeError> for DecompileError {
    fn from(error: EncodeError) -> Self {
        Self::EncodeError(error)
    }
}

impl From<DecodeError> for DecompileError {
    fn from(error: DecodeError) -> Self {
        Self::DecodeError(error)
    }
}

impl From<fmt::Error> for DecompileError {
    fn from(error: fmt::Error) -> Self {
        Self::FormattingError(error)
    }
}

impl From<RustToManifestValueError> for DecompileError {
    fn from(error: RustToManifestValueError) -> Self {
        Self::ValueConversionError(error)
    }
}

#[derive(Default)]
pub struct DecompilationContext<'a> {
    pub address_bech32_encoder: Option<&'a AddressBech32Encoder>,
    pub transaction_hash_bech32_encoder: Option<&'a TransactionHashBech32Encoder>,
    pub id_allocator: ManifestIdAllocator,
    pub object_names: ManifestObjectNamesRef<'a>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(
        address_bech32_encoder: &'a AddressBech32Encoder,
        transaction_hash_bech32_encoder: &'a TransactionHashBech32Encoder,
        object_names: ManifestObjectNamesRef<'a>,
    ) -> Self {
        Self {
            address_bech32_encoder: Some(address_bech32_encoder),
            transaction_hash_bech32_encoder: Some(transaction_hash_bech32_encoder),
            object_names,
            ..Default::default()
        }
    }

    pub fn for_value_display(&'a self) -> ManifestDecompilationDisplayContext<'a> {
        ManifestDecompilationDisplayContext::with_bech32_and_names(
            self.address_bech32_encoder,
            self.object_names,
        )
        .with_multi_line(4, 4)
    }

    pub fn transaction_hash_encoder(&'a self) -> Option<&'a TransactionHashBech32Encoder> {
        self.transaction_hash_bech32_encoder
    }

    pub fn new_bucket(&mut self) -> ManifestBucket {
        self.id_allocator.new_bucket_id()
    }

    pub fn new_proof(&mut self) -> ManifestProof {
        self.id_allocator.new_proof_id()
    }

    pub fn new_address_reservation(&mut self) -> ManifestAddressReservation {
        self.id_allocator.new_address_reservation_id()
    }

    pub fn new_address(&mut self) -> ManifestAddress {
        let id = self.id_allocator.new_address_id();
        ManifestAddress::Named(id)
    }

    pub fn new_named_intent(&mut self) -> ManifestNamedIntent {
        self.id_allocator.new_named_intent_id()
    }

    /// Allocate addresses before transaction, for system transactions only.
    pub fn preallocate_addresses(&mut self, n: u32) {
        for _ in 0..n {
            let _ = self.new_address();
        }
    }
}

pub fn decompile_any(
    manifest: &AnyManifest,
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    match manifest {
        AnyManifest::V1(m) => decompile(m, network),
        AnyManifest::SystemV1(m) => decompile(m, network),
        AnyManifest::V2(m) => decompile(m, network),
        AnyManifest::SubintentV2(m) => decompile(m, network),
    }
}

/// Contract: if the instructions are from a validated notarized transaction, no error
/// should be returned.
pub fn decompile(
    manifest: &impl TypedReadableManifest,
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    let address_bech32_encoder = AddressBech32Encoder::new(network);
    let transaction_hash_encoder = TransactionHashBech32Encoder::new(network);
    let mut buf = String::new();
    let mut context = DecompilationContext::new(
        &address_bech32_encoder,
        &transaction_hash_encoder,
        manifest.get_known_object_names_ref(),
    );
    for preallocated_address in manifest.get_preallocated_addresses() {
        let psuedo_instruction =
            preallocated_address.decompile_as_pseudo_instruction(&mut context)?;
        output_instruction(&mut buf, &context, psuedo_instruction)?;
    }
    for child_subintent in manifest.get_child_subintent_hashes() {
        let psuedo_instruction = child_subintent.decompile_as_pseudo_instruction(&mut context)?;
        output_instruction(&mut buf, &context, psuedo_instruction)?;
    }
    for inst in manifest.get_typed_instructions() {
        let decompiled = inst.decompile(&mut context)?;
        output_instruction(&mut buf, &context, decompiled)?;
    }

    Ok(buf)
}

pub struct DecompiledInstruction {
    command: &'static str,
    fields: Vec<DecompiledInstructionField>,
}

enum DecompiledInstructionField {
    Value(ManifestValue),
    Raw(String),
}

impl DecompiledInstruction {
    pub fn new(instruction: &'static str) -> Self {
        Self {
            command: instruction,
            fields: vec![],
        }
    }

    pub fn add_value_argument(mut self, value: ManifestValue) -> Self {
        self.fields.push(DecompiledInstructionField::Value(value));
        self
    }

    pub fn add_separated_tuple_value_arguments(
        mut self,
        tuple_args: &ManifestValue,
    ) -> Result<Self, DecompileError> {
        if let Value::Tuple { fields } = tuple_args {
            for argument in fields.iter() {
                self = self.add_value_argument(argument.clone());
            }
        } else {
            return Err(DecompileError::InvalidArguments);
        }
        Ok(self)
    }

    pub fn add_argument(self, value: impl ManifestEncode) -> Self {
        let encoded = manifest_encode(&value).unwrap();
        let value = manifest_decode(&encoded).unwrap();
        self.add_value_argument(value)
    }

    /// Only for use in pseudo-instructions.
    /// When we update the manifest value model, we should be able to discard these.
    pub fn add_raw_argument(mut self, value: String) -> Self {
        self.fields.push(DecompiledInstructionField::Raw(value));
        self
    }
}

pub fn output_instruction<F: fmt::Write>(
    f: &mut F,
    context: &DecompilationContext,
    DecompiledInstruction {
        command,
        fields: arguments,
    }: DecompiledInstruction,
) -> Result<(), DecompileError> {
    let value_display_context = context.for_value_display();
    write!(f, "{}", command)?;
    for argument in arguments.iter() {
        write!(f, "\n")?;
        match argument {
            DecompiledInstructionField::Value(value) => {
                format_manifest_value(f, value, &value_display_context, true, 0)?;
            }
            DecompiledInstructionField::Raw(raw_argument) => {
                let initial_indent = value_display_context.get_indent(0);
                write!(f, "{initial_indent}{raw_argument}")?;
            }
        }
    }
    if arguments.len() > 0 {
        write!(f, "\n;\n")?;
    } else {
        write!(f, ";\n")?;
    }

    Ok(())
}
