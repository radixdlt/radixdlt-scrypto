use colored::*;
use radix_engine::model::*;
use std::iter;

use crate::transaction::*;

pub trait IdentifyLast: Iterator + Sized {
    fn identify_last(self) -> Iter<Self>;
}

impl<I: Iterator> IdentifyLast for I {
    fn identify_last(self) -> Iter<Self> {
        Iter(self.peekable())
    }
}

pub struct Iter<I: Iterator>(iter::Peekable<I>);

impl<I: Iterator> Iterator for Iter<I> {
    type Item = (bool, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| (self.0.peek().is_none(), e))
    }
}

pub fn dump_receipt(receipt: TransactionReceipt) {
    println!(
        "\n{} {}",
        "Transaction Status:".bold().green(),
        if receipt.success {
            "SUCCESS".blue()
        } else {
            "FAILURE".red()
        }
        .bold()
    );

    println!("\n{}", "Instructions:".bold().green());
    for (last, inst) in receipt.transaction.instructions.iter().identify_last() {
        println!("{} {:02x?}", prefix(last), inst);
    }

    println!("\n{}", "Results:".bold().green());
    for (last, result) in receipt.results.iter().identify_last() {
        println!("{} {:02x?}", prefix(last), result);
    }

    println!("\n{}", "Logs:".bold().green());
    for (last, (level, msg)) in receipt.logs.iter().identify_last() {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };
        println!("{} [{:5}] {}", prefix(last), l, m);
    }

    println!(
        "\n{} {} ms\n",
        "Execution Time:".bold().green(),
        receipt.execution_time
    );
}

fn prefix(last: bool) -> &'static str {
    if last {
        "└─"
    } else {
        "├─"
    }
}
