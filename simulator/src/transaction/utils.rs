use colored::*;
use scrypto::types::*;

use crate::transaction::*;

pub fn print_receipt(receipt: TransactionReceipt) {
    for i in 0..receipt.transaction.actions.len() {
        println!("Action: {:?}", receipt.transaction.actions[i]);
        match receipt.results.get(i) {
            Some(r) => {
                println!("Result: {:02x?}", r);
            }
            None => {
                println!("Skipped");
            }
        }
    }

    for (level, msg) in receipt.logs {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };
        println!("  [{:5}] {}", l, m);
    }
}
