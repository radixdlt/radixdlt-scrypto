use clap::Parser;
use radix_engine::transaction::*;
use radix_engine::wasm::*;
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

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Transfer {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut wasm_engine = default_wasm_engine();
        let mut executor =
            TransactionExecutor::new(&mut substate_store, &mut wasm_engine, self.trace);
        let transaction = TransactionBuilder::new()
            .withdraw_from_account_by_amount(
                self.amount,
                self.resource_address,
                get_default_account()?,
            )
            .call_method_with_all_resources(self.recipient, "deposit_batch")
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
