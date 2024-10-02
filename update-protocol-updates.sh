set -ex

cargo test --package radix-transaction-scenarios --lib -- runners::dumper::test::update_all_generated_protocol_update_receipts --exact --ignored --nocapture
