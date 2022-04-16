use clap::Parser;
use radix_engine::transaction::*;
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

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Mint {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let default_account = get_default_account()?;
        let (default_pk, default_sk) = get_default_signers()?;

        let transaction = TransactionBuilder::new()
            .create_proof_from_account(self.minter_resource_address, default_account)
            .mint(
                self.amount,
                self.resource_address,
            )
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build(executor.get_nonce([default_pk]))
            .sign([&default_sk]);
        process_transaction(transaction, &mut executor, &self.manifest)
    }
}
