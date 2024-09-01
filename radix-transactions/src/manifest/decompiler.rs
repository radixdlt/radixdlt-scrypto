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
    pub id_allocator: ManifestIdAllocator,
    pub object_names: ManifestObjectNames,
}

#[derive(Default, Clone, Debug)]
pub struct ManifestObjectNames {
    pub bucket_names: NonIterMap<ManifestBucket, String>,
    pub proof_names: NonIterMap<ManifestProof, String>,
    pub address_reservation_names: NonIterMap<ManifestAddressReservation, String>,
    pub address_names: NonIterMap<u32, String>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(
        address_bech32_encoder: &'a AddressBech32Encoder,
        object_names: ManifestObjectNames,
    ) -> Self {
        Self {
            address_bech32_encoder: Some(address_bech32_encoder),
            object_names,
            ..Default::default()
        }
    }

    pub fn new_with_optional_network(
        address_bech32_encoder: Option<&'a AddressBech32Encoder>,
    ) -> Self {
        Self {
            address_bech32_encoder,
            ..Default::default()
        }
    }

    pub fn for_value_display(&'a self) -> ManifestDecompilationDisplayContext<'a> {
        ManifestDecompilationDisplayContext::with_bech32_and_names(
            self.address_bech32_encoder,
            &self.object_names.bucket_names,
            &self.object_names.proof_names,
            &self.object_names.address_reservation_names,
            &self.object_names.address_names,
        )
        .with_multi_line(4, 4)
    }

    pub fn new_bucket(&mut self) -> ManifestBucket {
        let id = self.id_allocator.new_bucket_id();
        if !self.object_names.bucket_names.contains_key(&id) {
            let name = format!("bucket{}", self.object_names.bucket_names.len() + 1);
            self.object_names.bucket_names.insert(id, name);
        }
        id
    }

    pub fn new_proof(&mut self) -> ManifestProof {
        let id = self.id_allocator.new_proof_id();
        if !self.object_names.proof_names.contains_key(&id) {
            let name = format!("proof{}", self.object_names.proof_names.len() + 1);
            self.object_names.proof_names.insert(id, name);
        }
        id
    }

    pub fn new_address_reservation(&mut self) -> ManifestAddressReservation {
        let id = self.id_allocator.new_address_reservation_id();
        if !self
            .object_names
            .address_reservation_names
            .contains_key(&id)
        {
            let name = format!(
                "reservation{}",
                self.object_names.address_reservation_names.len() + 1
            );
            self.object_names.address_reservation_names.insert(id, name);
        }
        id
    }

    pub fn new_address(&mut self) -> ManifestAddress {
        let id = self.id_allocator.new_address_id();
        if !self.object_names.address_names.contains_key(&id) {
            let name = format!("address{}", self.object_names.address_names.len() + 1);
            self.object_names.address_names.insert(id, name);
        }
        ManifestAddress::Named(id)
    }

    /// Allocate addresses before transaction, for system transactions only.
    pub fn preallocate_addresses(&mut self, n: u32) {
        for _ in 0..n {
            let _ = self.new_address();
        }
    }
}

/// Contract: if the instructions are from a validated notarized transaction, no error
/// should be returned.
pub fn decompile(
    instructions: &[impl InstructionVersion],
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    decompile_with_known_naming(instructions, network, Default::default())
}

pub fn decompile_with_known_naming(
    instructions: &[impl InstructionVersion],
    network: &NetworkDefinition,
    known_object_names: ManifestObjectNames,
) -> Result<String, DecompileError> {
    let address_bech32_encoder = AddressBech32Encoder::new(network);
    let mut buf = String::new();
    let mut context = DecompilationContext::new(&address_bech32_encoder, known_object_names);
    for inst in instructions {
        let decompiled = inst.decompile(&mut context)?;
        output_instruction(&mut buf, &context, decompiled)?;
    }

    Ok(buf)
}

pub struct DecompiledInstruction {
    command: &'static str,
    fields: Vec<ManifestValue>,
}

impl DecompiledInstruction {
    pub fn new(instruction: &'static str) -> Self {
        Self {
            command: instruction,
            fields: vec![],
        }
    }

    pub fn add_value_argument(mut self, value: ManifestValue) -> Self {
        self.fields.push(value);
        self
    }

    pub fn add_argument(self, value: impl ManifestEncode) -> Self {
        let encoded = manifest_encode(&value).unwrap();
        let value = manifest_decode(&encoded).unwrap();
        self.add_value_argument(value)
    }
}

pub fn output_instruction<F: fmt::Write>(
    f: &mut F,
    context: &DecompilationContext,
    DecompiledInstruction { command, fields }: DecompiledInstruction,
) -> Result<(), DecompileError> {
    write!(f, "{}", command)?;
    for field in fields.iter() {
        write!(f, "\n")?;
        format_manifest_value(f, field, &context.for_value_display(), true, 0)?;
    }
    if fields.len() > 0 {
        write!(f, "\n;\n")?;
    } else {
        write!(f, ";\n")?;
    }

    Ok(())
}
