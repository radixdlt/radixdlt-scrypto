use clap::Parser;
use radix_common::prelude::*;
use radix_transactions::builder::ManifestBuilder;

use crate::resim::*;
use crate::utils::*;

/// Call a function
#[derive(Parser, Debug)]
pub struct CallFunction {
    /// The package which the function belongs to
    pub package_address: SimulatorPackageAddress,

    /// The name of the blueprint which the function belongs to
    pub blueprint_name: String,

    /// The function name
    pub function_name: String,

    /// The call arguments, such as "5", "hello", "<resource_address>:<amount>" and "<resource_address>:<nf_local_id1>,<nf_local_id2>"
    pub arguments: Vec<String>,

    /// The proofs to add to the auth zone, in form of "<resource_address>:<amount>" or "<resource_address>:<nf_local_id1>,<nf_local_id2>"
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

impl CallFunction {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
        let address_bech32_decoder = AddressBech32Decoder::for_simulator();

        let default_account = get_default_account()?;
        let proofs = self.proofs.clone().unwrap_or_default();

        let mut builder = ManifestBuilder::new();
        builder = builder.lock_fee_from_faucet();
        for resource_specifier in proofs {
            builder = create_proof_from_account(
                builder,
                &address_bech32_decoder,
                default_account,
                resource_specifier,
            )
            .map_err(Error::FailedToBuildArguments)?;
        }
        let manifest = self
            .add_call_function_instruction_with_schema(
                builder,
                &address_bech32_decoder,
                self.package_address.0,
                self.blueprint_name.clone(),
                self.function_name.clone(),
                self.arguments.clone(),
                Some(default_account),
            )?
            .try_deposit_entire_worktop_or_refund(default_account, None)
            .build();
        handle_manifest(
            manifest.into(),
            &self.signing_keys,
            &self.network,
            &self.manifest,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
    }

    /// Calls a function.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// function SCHEMA, including resource buckets and proofs.
    ///
    /// If an Account component address is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction worktop.
    pub fn add_call_function_instruction_with_schema<'a>(
        &self,
        builder: ManifestBuilder,
        address_bech32_decoder: &AddressBech32Decoder,
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: Vec<String>,
        account: Option<ComponentAddress>,
    ) -> Result<ManifestBuilder, Error> {
        let bp_interface = export_blueprint_interface(package_address, &blueprint_name)?;

        let function_schema = bp_interface
            .find_function(function_name.as_str())
            .ok_or_else(|| {
                Error::TransactionConstructionError(BuildCallInstructionError::FunctionNotFound(
                    function_name.clone(),
                ))
            })?;

        let (schema, index) = match function_schema.input {
            BlueprintPayloadDef::Static(ScopedTypeId(hash, index)) => {
                let schema = export_schema(package_address.as_node_id(), hash)?;
                (schema, index)
            }
            BlueprintPayloadDef::Generic(_instance_index) => {
                todo!()
            }
        };

        let (builder, built_args) = build_call_arguments(
            builder,
            address_bech32_decoder,
            &schema,
            index,
            args,
            account,
        )
        .map_err(|e| {
            Error::TransactionConstructionError(BuildCallInstructionError::FailedToBuildArguments(
                e,
            ))
        })?;

        Ok(builder.call_function_raw(package_address, blueprint_name, function_name, built_args))
    }
}
