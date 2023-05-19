use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::WasmInstrumenter;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine::vm::ScryptoVm;
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::dec;
use radix_engine_interface::rule;
use radix_engine_store_interface::interface::{SubstateDatabase, DbPartitionKey, DbSortKey, DbSubstateValue, PartitionEntry, CommittableSubstateDatabase, DatabaseUpdates};
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::TestTransaction;
use std::path::PathBuf;
use std::time::Duration;
use radix_engine_stores::rocks_db::RocksdbSubstateStore;
use std::fs::File;
use std::io::prelude::*;
use plotters::prelude::*;
use linreg::linear_regression_of;


struct RocksdbSubstateStoreWithMetrics {
    db: RocksdbSubstateStore,
    read_metrics: RefCell<BTreeMap<usize, Vec<Duration>>>
}

impl RocksdbSubstateStoreWithMetrics {
    pub fn new(path: PathBuf) -> Self {
        Self {
            db: RocksdbSubstateStore::standard(path),
            read_metrics: RefCell::new(BTreeMap::new())
        }
    }

    #[allow(dead_code)]
    pub fn show_output(&self) {
        for (k, v) in self.read_metrics.borrow().iter() {
            println!("{:<10} | {:<10?}", k, v);
        }
    }

    #[allow(dead_code)]
    pub fn export_to_csv(&self) -> std::io::Result<()> {
        let mut file = File::create("/tmp/out_01.csv")?;

        file.write_all(b"Size;Duration[ns]\n")?;

        for (k, v) in self.read_metrics.borrow().iter() {
            for i in v {
                file.write_all(format!("{};{}\n", k, i.as_nanos()).as_bytes())?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn export_histogram(&self) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new("/tmp/h1.png", (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .margin(5)
            .caption("Histogram Test", ("sans-serif", 50.0))
            .build_cartesian_2d((0u32..2000u32).into_segmented(), 900u32..10000u32)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(&WHITE.mix(0.3))
            .y_desc("Count")
            .x_desc("Bucket")
            .axis_desc_style(("sans-serif", 15))
            .draw()?;

        let mut data = Vec::with_capacity(100000);
        for (k, v) in self.read_metrics.borrow().iter() {
            for i in v {
                data.push((*k as u32, i.as_nanos() as u32));
            }
        }

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(RED.mix(0.5).filled())
                .data(data),
        )?;

        root.present().expect("Unable to write result to file");

        Ok(())
    }


    pub fn export_mft(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. calculate max values
        let mut max_values = Vec::with_capacity(100000);
        let binding = self.read_metrics.borrow();
        for (_k, v) in binding.iter() {
            max_values.push( v.iter().max().unwrap() );
        }

        // 2. filter out spikes and calculate medians
        //let peak_diff_division = 10;
        //let mut idx = 0;
        let mut data = Vec::with_capacity(100000);
        let mut median_data = Vec::new();
        for (k, v) in self.read_metrics.borrow().iter() {
            let mut w = v.iter().map(|i| *i).collect();
            let median = calculate_median(&mut w);
            // if *max_values[idx] > 10 * median {
            //     let max_spike_offset = Duration::from_nanos((max_values[idx].as_nanos() / peak_diff_division) as u64);
            //     discard_spikes(&mut w, max_spike_offset);
            // }
            for i in w {
                data.push((*k as i32, i.as_nanos() as i32));
            }
            median_data.push((*k as i32, median.as_nanos() as i32));
            //idx += 1;
        }

        // 3. calculate axis max/min values
        let y_ofs = 1000;
        let x_ofs = 5000;
        let x_min = data.iter().map(|i| i.0).min().unwrap() - x_ofs;
        let x_max = data.iter().map(|i| i.0).max().unwrap() + x_ofs;
        let y_min = data.iter().map(|i| i.1).min().unwrap() - y_ofs;
        let y_max = data.iter().map(|i| i.1).max().unwrap() + y_ofs;

        // 4. calculate linear approximation
        let (lin_slope, lin_intercept): (f64, f64) = linear_regression_of(&data).unwrap();
        let lin_x_axis = (x_min..x_max).step(10);

        // draw scatter plot
        let root = BitMapBackend::new("/tmp/h2.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;
        root.margin(20, 20, 20, 20);

        let mut scatter_ctx = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .margin(20)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
        scatter_ctx.configure_mesh()
            .x_desc("Size [bytes]")
            .y_desc("DB read duration [nanoseconds]")
            .axis_desc_style(("sans-serif", 16))
            .draw()?;
        // 1. draw all read points
        scatter_ctx.draw_series(
            data
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?
            .label(format!("Reads (count: {})", data.len()))
            .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
        // 2. draw median for each read series (basaed on same size)
        scatter_ctx.draw_series(
            median_data
                .iter()
                .map(|(x, y)| Cross::new((*x, *y), 6, RED)),
            )?
            .label("Median for each series")
            .legend(|(x, y)| Cross::new((x + 10, y), 6, RED));
        // 3. draw linear approximetion line
        scatter_ctx.draw_series(LineSeries::new(
                lin_x_axis.values().map(|x| (x, (lin_slope * x as f64 + lin_intercept) as i32)),
                &BLUE,
            ))?
            .label(format!("Linear approx.: f(x)={:.4}*x+{:.1}", lin_slope, lin_intercept))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        scatter_ctx.configure_series_labels()
                .background_style(&WHITE)
                .border_style(&BLACK)
                .label_font(("sans-serif", 16))
                .position(SeriesLabelPosition::UpperMiddle
            )
            .draw()?;

        root.present().expect("Unable to write result to file");

        // print some informations
        println!("Read count: {}", data.len());
        println!("Distinct size read count: {}", self.read_metrics.borrow().len());
        println!("Read counts list (size, count): {:?}", self.read_metrics.borrow().iter().map(|(k,v)| (*k, v.len()) ).collect::<Vec<(usize, usize)>>());
        println!("Median points list (size, time): {:?}", median_data);
        println!("Linear approx.:  f(size) = {} * size + {}", lin_slope, lin_intercept);

        Ok(())
    }

    /*pub fn export_mft(&self) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new("/tmp/h2.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;
    
        let mut data = Vec::with_capacity(100000);
        for (k, v) in self.read_metrics.borrow().iter() {
            for i in v {
                data.push((*k as u32, i.as_nanos() as u32));
            }
        }
        let x_min = data.iter().map(|i| i.0).min().unwrap();
        let x_max = data.iter().map(|i| i.0).max().unwrap();
        let y_min = data.iter().map(|i| i.1).min().unwrap();
        let y_max = data.iter().map(|i| i.1).max().unwrap();
    
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .caption("MSFT Stock Price", ("sans-serif", 50.0).into_font())
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    
        chart.configure_mesh().light_line_style(&WHITE).draw()?;
    
        chart.draw_series(
            data.iter().map(|x| {
                CandleStick::new(x.0, x.1, x.2, x.3, x.4, GREEN.filled(), RED, 15)
//                CandleStick::new(parse_time(x.0), x.1, x.2, x.3, x.4, GREEN.filled(), RED, 15)
            }),
        )?;
    
        root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    
        Ok(())
    }*/
}

impl SubstateDatabase for RocksdbSubstateStoreWithMetrics {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {

        let start = std::time::Instant::now();
        let ret = self.db.get_substate(partition_key,sort_key);
        let duration = start.elapsed();

        if let Some(value) = ret {
            let exists = self.read_metrics.borrow().get(&value.len()).is_some();
            if exists {
                self.read_metrics.borrow_mut().get_mut(&value.len()).unwrap().push(duration);
            } else {
                self.read_metrics.borrow_mut().insert(value.len(), vec![duration]);
            }
            Some(value)
        } else {
            None
        }
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.db.list_entries(partition_key)
    }
}

impl CommittableSubstateDatabase for RocksdbSubstateStoreWithMetrics {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        self.db.commit(database_updates)
    }
}


fn db_rw_test(c: &mut Criterion) {
    println!("starting");
    // Set up environment.
    let mut scrypto_interpreter = ScryptoVm {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::V0,
    };

    let path = PathBuf::from(r"/tmp/radix-scrypto-db");
    // clean database
    std::fs::remove_dir_all(path.clone()).ok();

    let mut substate_db = RocksdbSubstateStoreWithMetrics::new(path);
    Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false)
        .bootstrap_test_default()
        .unwrap();

    // Create a key pair
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let accounts = (0..2)
        .map(|_| {
            let config = AccessRulesConfig::new().default(
                rule!(require(NonFungibleGlobalId::from_public_key(&public_key))),
                rule!(require(NonFungibleGlobalId::from_public_key(&public_key))),
            );
            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET, 100.into())
                .new_account_advanced(config)
                .build();
            let account = execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            )
            .expect_commit(true)
            .new_component_addresses()[0];

            account
        })
        .collect::<Vec<ComponentAddress>>();

    let account1 = accounts[0];
    let account2 = accounts[1];

    // Fill first account
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100.into())
        .call_method(FAUCET, "free", manifest_args!())
        .call_method(
            account1,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    for nonce in 0..10000 {
        execute_and_commit_transaction(
            &mut substate_db,
            &mut scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
        )
        .expect_commit(true);
    }

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100.into())
        .withdraw_from_account(account1, RADIX_TOKEN, dec!("0.000001"))
        .call_method(
            account2,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("Transfer::run", |b| {
        b.iter(|| {
            let receipt = execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            );
            receipt.expect_commit_success();
            nonce += 1;
        })
    });

    //substate_db.show_output();
    //substate_db.export_to_csv().unwrap();
    substate_db.export_mft().unwrap();
}

criterion_group!(database, db_rw_test);
criterion_main!(database);


fn calculate_median(data: &mut Vec<Duration>) -> Duration {
    data.sort();
    let center_idx = data.len() / 2;
    let median = data[center_idx];
    median
}

#[allow(dead_code)]
fn discard_spikes(data: &mut Vec<Duration>, delta_range: Duration) {
    // 1. calculate median
    let median = calculate_median(data);

    // 2. discard items out of median + range
    data.retain(|&i| {
        if i > median {
            i - median <= delta_range
        } else {
            median - i <= delta_range
        }
    });
}
