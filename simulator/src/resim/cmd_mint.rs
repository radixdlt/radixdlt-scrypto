use clap::Parser;
use radix_engine::transaction::*;
use radix_engine::wasm::*;
use scrypto::engine::types::*;

use crate::resim::*;

/// Mint resource
#[derive(Parser, Debug)]
pub struct Mint {
    /// The amount of resource to mint
    amount: Decimal,

    /// The resource address
    resource_address: ResourceAddress,

    /// The minter resource address
    minter_resource_address: ResourceAddress,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Mint {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut substate_store = RadixEngineDB::new(get_data_dir()?);
        let mut wasm_engine = default_wasm_engine();
        let mut executor =
            TransactionExecutor::new(&mut substate_store, &mut wasm_engine, self.trace);
        let default_account = get_default_account()?;

        let transaction = TransactionBuilder::new()
            .create_proof_from_account(self.minter_resource_address, default_account)
            .mint(self.amount, self.resource_address)
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build_with_no_nonce();
        process_transaction(
            &mut executor,
            transaction,
            &self.signing_keys,
            &self.manifest,
            out,
        )
    }
}
