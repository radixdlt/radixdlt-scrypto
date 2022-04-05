use clap::Parser;
use radix_engine::transaction::*;
use scrypto::engine::types::*;

use crate::resim::*;

/// Transfer resource to another account
#[derive(Parser, Debug)]
pub struct Transfer {
    /// The amount to transfer.
    amount: Decimal,

    /// The resource address.
    resource_address: ResourceAddress,

    /// The recipient component address.
    recipient: ComponentAddress,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Transfer {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let default_account = get_default_account()?;
        let (default_pks, default_sks) = get_default_signers()?;
        let transaction = TransactionBuilder::new(&executor)
            .withdraw_from_account_by_amount(self.amount, self.resource_address, default_account)
            .call_method_with_all_resources(self.recipient, "deposit_batch")
            .build(default_pks)
            .map_err(Error::TransactionConstructionError)?
            .sign(&default_sks);
        process_transaction(transaction, &mut executor, &self.manifest)
    }
}
