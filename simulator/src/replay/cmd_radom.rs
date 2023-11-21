use super::Error;
use clap::Parser;
use diesel::Connection;
use diesel::PgConnection;
use radix_engine::types::*;
use rand::Rng;
use rand::RngCore;

#[derive(Parser, Debug)]
pub struct Random {}

impl Random {
    /*
       DROP TABLE event;

       CREATE TABLE event (
           uuid BIGSERIAL PRIMARY KEY,

           address VARCHAR NOT NULL,
           timestamp BIGINT,
           transaction_location VARCHAR,

           received_address VARCHAR,
           received_fungible FLOAT,
           received_non_fungible VARCHAR[],

           sent_address VARCHAR,
           sent_fungible FLOAT,
           sent_non_fungible VARCHAR[]
       );

       CREATE INDEX event_address_index ON event USING btree(address);
       CREATE INDEX event_timestamp_index ON event USING btree(timestamp);
       CREATE INDEX event_transaction_location_index ON event USING btree(transaction_location);
       CREATE INDEX event_received_address_index ON event USING btree(received_address);
       CREATE INDEX event_sent_address_index ON event USING btree(sent_address);
    */
    pub fn run(&self) -> Result<(), Error> {
        // Test environment, protected under VPC, no public exposure
        let connection = PgConnection::establish("postgres://postgres:radixdlt.com@aurora-serverless.cluster-cual2lka9k9x.eu-west-1.rds.amazonaws.com/receipts").unwrap();
        println!("Connection to the database established!");

        let mut rng = rand::thread_rng();
        let address_encoder = AddressBech32Encoder::new(&NetworkDefinition::mainnet());

        let mut accounts = vec![];
        for _ in 0..1_000_000 {
            let mut random = rng.gen::<[u8; NodeId::LENGTH]>();
            random[0] = EntityType::GlobalAccount as u8;
            accounts.push(address_encoder.encode(&random).unwrap());
        }

        let mut resources = vec![];
        for _ in 0..1_000 {
            let mut random = rng.gen::<[u8; NodeId::LENGTH]>();
            random[0] = EntityType::GlobalFungibleResourceManager as u8;
            resources.push(address_encoder.encode(&random).unwrap());
        }

        for timestamp in 0..100_000_000 {
            let address1 = &accounts[rng.next_u64() as usize % accounts.len()];
            let address2 = &accounts[rng.next_u64() as usize % accounts.len()];
            let resource = &resources[rng.next_u64() as usize % resources.len()];
            let amount = rng.next_u32();

            connection.execute(&format!("
                INSERT INTO event (address, timestamp, transaction_location, sent_address, sent_fungible)
                VALUES
                ('{address1}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount}),
                ('{address1}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount}),
                ('{address1}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount}),
                ('{address1}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount}),
                ('{address1}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount})
            ")).unwrap();
            connection.execute(&format!("
                INSERT INTO event (address, timestamp, transaction_location, received_address, received_fungible)
                VALUES
                ('{address2}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount}),
                ('{address2}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount}),
                ('{address2}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount}),
                ('{address2}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount}),
                ('{address2}', {timestamp}, 's3://super_bucket/{timestamp}.bin', '{resource}', {amount})
            ")).unwrap();
            if timestamp % 100 == 0 {
                println!("{}", timestamp);
            }
        }

        Ok(())
    }
}
