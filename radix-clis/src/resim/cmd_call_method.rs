#![allow(unused_must_use)]

use clap::Parser;
use radix_common::prelude::*;

use crate::resim::*;
use crate::utils::*;

/// Call a method
#[derive(Parser, Debug)]
pub struct CallMethod {
    /// The component that the method belongs to
    pub component_address: SimulatorComponentAddress,

    /// The method name
    pub method_name: String,

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

impl CallMethod {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
        let address_bech32_decoder = AddressBech32Decoder::for_simulator();

        let default_account = get_default_account()?;
        let proofs = self.proofs.clone().unwrap_or_default();

        let mut builder = ManifestBuilder::new().lock_fee_from_faucet();
        for resource_specifier in proofs {
            builder = create_proof_from_account(
                builder,
                &address_bech32_decoder,
                default_account,
                resource_specifier,
            )
            .map_err(Error::FailedToBuildArguments)?
        }

        let manifest = self
            .add_call_method_instruction_with_schema(
                builder,
                &address_bech32_decoder,
                self.component_address.0,
                self.method_name.clone(),
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

    /// Calls a method.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// method SCHEMA, including resource buckets and proofs.
    ///
    /// If an Account component address is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction worktop.
    pub fn add_call_method_instruction_with_schema<'a>(
        &self,
        builder: ManifestBuilder,
        address_bech32_decoder: &AddressBech32Decoder,
        component_address: ComponentAddress,
        method_name: String,
        args: Vec<String>,
        account: Option<ComponentAddress>,
    ) -> Result<ManifestBuilder, Error> {
        let object_info = export_object_info(component_address)?;
        let bp_info = object_info.blueprint_info;
        let bp_id = bp_info.blueprint_id;
        let bp_interface =
            export_blueprint_interface(bp_id.package_address, &bp_id.blueprint_name)?;

        let function_schema = bp_interface
            .find_method(method_name.as_str())
            .ok_or_else(|| {
                Error::TransactionConstructionError(BuildCallInstructionError::MethodNotFound(
                    method_name.clone(),
                ))
            })?;

        let (schema, index) = match function_schema.input {
            BlueprintPayloadDef::Static(ScopedTypeId(schema_hash, index)) => {
                let schema = export_schema(bp_id.package_address.as_node_id(), schema_hash)?;
                (schema, index)
            }
            BlueprintPayloadDef::Generic(generic_index) => {
                let type_subst_ref = bp_info
                    .generic_substitutions
                    .get(generic_index as usize)
                    .ok_or_else(|| Error::InstanceSchemaNot(component_address, generic_index))?;

                match type_subst_ref {
                    GenericSubstitution::Local(type_id) => {
                        let schema = export_schema(bp_id.package_address.as_node_id(), type_id.0)?;
                        (schema, type_id.1)
                    }
                    GenericSubstitution::Remote(_) => {
                        return Err(Error::RemoteGenericSubstitutionNotSupported);
                    }
                }
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

        Ok(builder.call_method_raw(component_address, method_name, built_args))
    }
}
