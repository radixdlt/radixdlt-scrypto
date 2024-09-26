use scrypto_bindgen::schema;
use scrypto_bindgen::translation;
use scrypto_bindgen::types;

use clap::Parser;
use radix_common::prelude::*;
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_substate_store_interface::interface::SubstateDatabase;
use std::io::Write;

use crate::resim::*;

use self::schema::*;

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

    #[clap(short, long)]
    func_sig_change: Vec<types::FunctionSignatureReplacementsInput>,
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

    let env = if args.reset_ledger {
        SimulatorEnvironment::new_reset().map_err(Error::ResimError)?
    } else {
        SimulatorEnvironment::new().map_err(Error::ResimError)?
    };
    let db = env.db;

    let blueprint_replacement_map = types::prepare_replacement_map(&args.func_sig_change);

    // Decode the package address without network context.
    let package_address = {
        let (_, _, bytes) =
            AddressBech32Decoder::validate_and_decode_ignore_hrp(&args.package_address)
                .map_err(Error::Bech32DecodeError)?;
        PackageAddress::try_from(bytes.as_slice()).map_err(Error::PackageAddressError)?
    };

    // Generating the bindings
    let bindings = {
        let reader = SystemDatabaseReader::new(&db);
        let definition = reader.get_package_definition(package_address);
        let schema_resolver = SchemaResolver::new(package_address, &db);

        let package_interface =
            schema::package_interface_from_package_definition(definition, &schema_resolver)
                .map_err(Error::SchemaError)?;
        let mut ast_package_interface = translation::package_schema_interface_to_ast_interface(
            package_interface,
            package_address,
            &schema_resolver,
            &blueprint_replacement_map,
        )
        .map_err(Error::SchemaError)?;

        // Scrypto-bindgen does not generate the aux-types. Only ledger-tools does.
        ast_package_interface.auxiliary_types = Default::default();

        ast_package_interface
    };

    writeln!(&mut out, "{}", quote::quote!(#bindings)).map_err(Error::IOError)?;

    Ok(())
}

pub struct SchemaResolver<'s, S>(PackageAddress, SystemDatabaseReader<'s, S>)
where
    S: SubstateDatabase;

impl<'s, S> SchemaResolver<'s, S>
where
    S: SubstateDatabase,
{
    pub fn new(node_id: PackageAddress, substate_database: &'s S) -> Self {
        let reader = SystemDatabaseReader::new(substate_database);
        Self(node_id, reader)
    }
}

impl<'s, S> PackageSchemaResolver for SchemaResolver<'s, S>
where
    S: SubstateDatabase,
{
    fn lookup_schema(&self, schema_hash: &SchemaHash) -> Option<Rc<VersionedScryptoSchema>> {
        self.1.get_schema(self.0.as_node_id(), schema_hash).ok()
    }

    fn resolve_type_kind(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<LocalTypeKind<ScryptoCustomSchema>, schema::SchemaError> {
        self.lookup_schema(&type_identifier.0)
            .ok_or(schema::SchemaError::FailedToGetSchemaFromSchemaHash)?
            .as_latest_version()
            .ok_or(schema::SchemaError::FailedToGetSchemaFromSchemaHash)?
            .resolve_type_kind(type_identifier.1)
            .ok_or(schema::SchemaError::NonExistentLocalTypeIndex(
                type_identifier.1,
            ))
            .cloned()
    }

    fn resolve_type_metadata(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<TypeMetadata, schema::SchemaError> {
        self.lookup_schema(&type_identifier.0)
            .ok_or(schema::SchemaError::FailedToGetSchemaFromSchemaHash)?
            .as_latest_version()
            .ok_or(schema::SchemaError::FailedToGetSchemaFromSchemaHash)?
            .resolve_type_metadata(type_identifier.1)
            .ok_or(schema::SchemaError::NonExistentLocalTypeIndex(
                type_identifier.1,
            ))
            .cloned()
    }

    fn resolve_type_validation(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<TypeValidation<ScryptoCustomTypeValidation>, schema::SchemaError> {
        self.lookup_schema(&type_identifier.0)
            .ok_or(schema::SchemaError::FailedToGetSchemaFromSchemaHash)?
            .as_latest_version()
            .ok_or(schema::SchemaError::FailedToGetSchemaFromSchemaHash)?
            .resolve_type_validation(type_identifier.1)
            .ok_or(schema::SchemaError::NonExistentLocalTypeIndex(
                type_identifier.1,
            ))
            .cloned()
    }

    fn package_address(&self) -> PackageAddress {
        self.0
    }
}
