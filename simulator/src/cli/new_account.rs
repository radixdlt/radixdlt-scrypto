use clap::{crate_version, App, ArgMatches, SubCommand};
use radix_engine::execution::*;
use radix_engine::model::*;
use rand::RngCore;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::cli::*;
use crate::ledger::*;

/// Constructs a `new-account` subcommand.
pub fn make_new_account_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_ACCOUNT)
        .about("Creates an account.")
        .version(crate_version!())
}

/// Handles a `new-account` request.
pub fn handle_new_account<'a>(_matches: &ArgMatches<'a>) {
    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    if runtime.get_resource(Address::RadixToken).is_none() {
        let xrd = Resource::new(ResourceInfo {
            symbol: "xrd".to_owned(),
            name: "Radix".to_owned(),
            description: "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.".to_owned(),
            url: "https://tokens.radixdlt.com".to_owned(),
            icon_url: "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_owned(),
            minter: Some(Address::System),
            supply: None,
        });
        runtime.put_resource(Address::RadixToken, xrd);
    }

    // mocked key pair
    let mut data = [0u8; 33];
    rand::thread_rng().fill_bytes(&mut data);
    let address = Address::PublicKey(data);

    // account
    let mut account = Account::new();
    let bid = runtime.new_persisted_bid();
    account.insert_bucket(Address::RadixToken, bid);
    runtime.put_account(address, account);

    // bucket
    let bucket = Bucket::new(1_000_000.into(), Address::RadixToken);
    runtime.put_bucket(bid, bucket);

    // flush
    runtime.flush();

    println!("New account: {}", address);
}
