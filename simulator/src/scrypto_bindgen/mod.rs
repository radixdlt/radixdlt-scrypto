mod ast;
mod schema;
mod translation;

use clap::Parser;
use radix_engine::system::{bootstrap::*, system_db_reader::SystemDatabaseReader};
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_engine_store_interface::interface::SubstateDatabase;
use radix_engine_stores::rocks_db::*;
use std::io::Write;

use crate::resim::*;

use self::schema::*;
use self::translation::blueprint_schema_interface_to_ast_interface;

/// Generates interfaces for Scrypto packages to ease the use of external packages.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "scrypto-bindgen")]
pub struct Args {
    /// The address of the package to generate the bindings for.
    package_address: String,

    /// When enabled, the ledger will be cleared and bootstrapped again before being used to obtain
    /// the bindings.
    #[clap(short, long)]
    reset_ledger: bool,
}

#[derive(Debug)]
pub enum Error {
    Bech32DecodeError(AddressBech32DecodeError),
    PackageAddressError(ParsePackageAddressError),
    ResimError(crate::resim::Error),
    SchemaError(SchemaError),
    IOError(std::io::Error),
}

pub fn run() -> Result<(), Error> {
    let args = Args::parse();

    // Everything will be written to the std-out
    let mut out = std::io::stdout();

    // Create the substate database
    let ledger_path = get_data_dir().map_err(Error::ResimError)?;
    let mut substate_db = RocksdbSubstateStore::standard(ledger_path);

    // Reset the ledger if required to.
    if args.reset_ledger {
        let mut buffer = Vec::new();
        crate::resim::Reset {}
            .run(&mut buffer)
            .map_err(Error::ResimError)?;

        let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
        let native_vm = DefaultNativeVm::new();
        let vm = Vm::new(&scrypto_vm, native_vm);
        Bootstrapper::new(
            NetworkDefinition::simulator(),
            &mut substate_db,
            vm.clone(),
            false,
        )
        .bootstrap_test_default();
    }

    // Decode the package address without network context.
    let package_address = {
        let (_, _, bytes) =
            AddressBech32Decoder::validate_and_decode_ignore_hrp(&args.package_address)
                .map_err(Error::Bech32DecodeError)?;
        PackageAddress::try_from(bytes.as_slice()).map_err(Error::PackageAddressError)?
    };

    // Generating the bindings
    let bindings = {
        let reader = SystemDatabaseReader::new(&substate_db);
        let definition = reader.get_package_definition(package_address);

        let schema_resolver = SchemaResolver::new(package_address, reader);
        derive_blueprint_interfaces(definition, &schema_resolver)
            .map_err(Error::SchemaError)?
            .into_iter()
            .map(|blueprint_interface| {
                blueprint_schema_interface_to_ast_interface(blueprint_interface, &schema_resolver)
                    .map(|blueprint_interface| blueprint_interface.to_token_stream(package_address))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::SchemaError)?
    };

    for binding in bindings {
        writeln!(&mut out, "{}", binding).map_err(Error::IOError)?
    }

    Ok(())
}

struct SchemaResolver<'s, S>(PackageAddress, SystemDatabaseReader<'s, S>)
where
    S: SubstateDatabase;

impl<'s, S> SchemaResolver<'s, S>
where
    S: SubstateDatabase,
{
    pub fn new(node_id: PackageAddress, reader: SystemDatabaseReader<'s, S>) -> Self {
        Self(node_id, reader)
    }
}

impl<'s, S> PackageSchemaResolver for SchemaResolver<'s, S>
where
    S: SubstateDatabase,
{
    fn lookup_schema(&self, schema_hash: &SchemaHash) -> Option<VersionedScryptoSchema> {
        self.1.get_schema(self.0.as_node_id(), schema_hash).ok()
    }

    fn resolve_type_kind(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<SchemaTypeKind<ScryptoCustomSchema>, schema::SchemaError> {
        self.lookup_schema(&type_identifier.0)
            .ok_or(SchemaError::FailedToGetSchemaFromSchemaHash)?
            .into_latest()
            .resolve_type_kind(type_identifier.1)
            .ok_or(SchemaError::NonExistentLocalTypeIndex(type_identifier.1))
            .cloned()
    }

    fn resolve_type_metadata(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<TypeMetadata, schema::SchemaError> {
        self.lookup_schema(&type_identifier.0)
            .ok_or(SchemaError::FailedToGetSchemaFromSchemaHash)?
            .into_latest()
            .resolve_type_metadata(type_identifier.1)
            .ok_or(SchemaError::NonExistentLocalTypeIndex(type_identifier.1))
            .cloned()
    }

    fn resolve_type_validation(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<TypeValidation<ScryptoCustomTypeValidation>, schema::SchemaError> {
        self.lookup_schema(&type_identifier.0)
            .ok_or(SchemaError::FailedToGetSchemaFromSchemaHash)?
            .into_latest()
            .resolve_type_validation(type_identifier.1)
            .ok_or(SchemaError::NonExistentLocalTypeIndex(type_identifier.1))
            .cloned()
    }

    fn package_address(&self) -> PackageAddress {
        self.0
    }
}
