use colored::*;
use radix_engine::model::*;

use crate::transaction::*;

pub fn print_receipt(receipt: TransactionReceipt) {
    for (i, action) in receipt.transaction.instructions.iter().enumerate() {
        println!("Instruction #{}: {:?}", i, action);
        match receipt.results.get(i) {
            Some(r) => {
                println!("Result: {:02x?}", r);
            }
            None => {
                println!("Skipped");
            }
        }
        println!();
    }

    for (level, msg) in receipt.logs {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };
        println!("[{:5}] {}", l, m);
    }
}
