use clap::Parser;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;

/// Transfer resource to another account
#[derive(Parser, Debug)]
pub struct Transfer {
    /// The resource to transfer, e.g. "amount,resource_address" or "#nft_id1,#nft_id2,resource_address"
    resource: ResourceAmount,

    /// The recipient address
    recipient: Address,

    /// The transaction signers
    #[clap(short, long)]
    signers: Vec<Address>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Transfer {
    pub fn run(&self) -> Result<(), Error> {
        let mut configs = get_configs()?;
        let account = configs.default_account.ok_or(Error::NoDefaultAccount)?;
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(
            &mut ledger,
            configs.current_epoch,
            configs.nonce,
            self.trace,
        );
        let transaction = TransactionBuilder::new(&executor)
            .withdraw_from_account(&self.resource, account.0)
            .call_method_with_all_resources(self.recipient, "deposit_batch")
            .build(self.signers.clone())
            .map_err(Error::TransactionConstructionError)?;
        let receipt = executor
            .run(transaction)
            .map_err(Error::TransactionValidationError)?;

        println!("{:?}", receipt);
        if receipt.result.is_ok() {
            configs.nonce = executor.nonce();
            set_configs(configs)?;
        }

        receipt.result.map_err(Error::TransactionExecutionError)
    }
}
