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
    pub object_names: ManifestObjectNamesRef<'a>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(
        address_bech32_encoder: &'a AddressBech32Encoder,
        object_names: ManifestObjectNamesRef<'a>,
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
            self.object_names,
        )
        .with_multi_line(4, 4)
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

    /// Allocate addresses before transaction, for system transactions only.
    pub fn preallocate_addresses(&mut self, n: u32) {
        for _ in 0..n {
            let _ = self.new_address();
        }
    }
}

pub fn decompile_any(
    manifest: &AnyTransactionManifest,
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    match manifest {
        AnyTransactionManifest::V1(m) => decompile(m, network),
        AnyTransactionManifest::SystemV1(m) => decompile(m, network),
        AnyTransactionManifest::V2(m) => decompile(m, network),
    }
}

/// Contract: if the instructions are from a validated notarized transaction, no error
/// should be returned.
pub fn decompile(
    manifest: &impl ReadableManifest,
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    let address_bech32_encoder = AddressBech32Encoder::new(network);
    let mut buf = String::new();
    let mut context = DecompilationContext::new(
        &address_bech32_encoder,
        manifest.get_known_object_names_ref(),
    );
    for inst in manifest.get_instructions() {
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
