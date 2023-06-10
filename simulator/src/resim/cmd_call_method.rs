#![allow(unused_must_use)]

use clap::Parser;
use radix_engine::types::*;
use transaction::builder::ManifestBuilder;

use crate::resim::*;
use crate::utils::*;

/// Call a method
#[derive(Parser, Debug)]
pub struct CallMethod {
    /// The component that the method belongs to
    pub component_address: SimulatorComponentAddress,

    /// The method name
    pub method_name: String,

    /// The call arguments, such as "5", "hello", "<amount>,<resource_address>" and "<resource_address>:<nf_local_id1>,<nf_local_id2>"
    pub arguments: Vec<String>,

    /// The proofs to add to the auth zone, in form of "<amount>,<resource_address>" or "<resource_address>:<nf_local_id1>,<nf_local_id2>"
    #[clap(short, long, multiple = true)]
    pub proofs: Option<Vec<String>>,

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    pub network: Option<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    pub manifest: Option<PathBuf>,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    pub signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
}

impl CallMethod {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let bech32_decoder = Bech32Decoder::for_simulator();

        let default_account = get_default_account()?;
        let proofs = self.proofs.clone().unwrap_or_default();

        let mut manifest_builder = &mut ManifestBuilder::new();
        for resource_specifier in proofs {
            manifest_builder = manifest_builder.borrow_mut(|builder| {
                create_proof_from_account(
                    builder,
                    &bech32_decoder,
                    default_account,
                    resource_specifier,
                )
                .map_err(Error::FailedToBuildArguments)?;
                Ok(builder)
            })?;
        }

        let blueprint = get_blueprint(self.component_address.0)?;

        let manifest = manifest_builder
            .lock_fee(FAUCET, 100.into())
            .borrow_mut(|builder| {
                self.add_call_method_instruction_with_schema(
                    builder,
                    &bech32_decoder,
                    self.component_address.0,
                    self.method_name.clone(),
                    self.arguments.clone(),
                    Some(default_account),
                    &export_blueprint_schema(blueprint.package_address, &blueprint.blueprint_name)?,
                )
                .map_err(Error::TransactionConstructionError)?;
                Ok(builder)
            })?
            .call_method(
                default_account,
                "try_deposit_batch_or_refund",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        handle_manifest(
            manifest,
            &self.signing_keys,
            &self.network,
            &self.manifest,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
    }

    /// Calls a method.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// method SCHEMA, including resource buckets and proofs.
    ///
    /// If an Account component address is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction worktop.
    pub fn add_call_method_instruction_with_schema<'a>(
        &self,
        builder: &'a mut ManifestBuilder,
        bech32_decoder: &Bech32Decoder,
        component_address: ComponentAddress,
        method_name: String,
        args: Vec<String>,
        account: Option<ComponentAddress>,
        blueprint_schema: &BlueprintDefinition,
    ) -> Result<&'a mut ManifestBuilder, BuildCallInstructionError> {
        let function_schema = blueprint_schema
            .find_method(method_name.as_str())
            .ok_or_else(|| BuildCallInstructionError::MethodNotFound(method_name.clone()))?;

        let index = match &function_schema.output {
            SchemaPointer::Package(_hash, index) => index.clone(),
        };

        let (builder, built_args) = build_call_arguments(
            builder,
            bech32_decoder,
            &blueprint_schema.schema,
            index,
            args,
            account,
        )?;

        builder.add_instruction(InstructionV1::CallMethod {
            address: component_address.into(),
            method_name,
            args: built_args,
        });
        Ok(builder)
    }
}
