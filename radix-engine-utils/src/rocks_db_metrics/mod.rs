use radix_engine_store_interface::interface::{
    CommittableSubstateDatabase, DatabaseUpdates, DbPartitionKey, DbSortKey, DbSubstateValue,
    PartitionEntry, SubstateDatabase,
};
use radix_engine_stores::{rocks_db::{BlockBasedOptions, Options, RocksdbSubstateStore}, memory_db::InMemorySubstateDatabase};
use std::{path::PathBuf, time::Duration, cell::RefCell, collections::BTreeMap};
use linreg::linear_regression_of;
use plotters::prelude::*;


pub struct SubstateStoreWithMetrics<S>
where
    S: SubstateDatabase + CommittableSubstateDatabase
{
    db: S,
    read_metrics: RefCell<BTreeMap<usize, Vec<Duration>>>,
}

impl SubstateStoreWithMetrics<RocksdbSubstateStore> {
    pub fn new_rocksdb(path: PathBuf) -> Self {
        let mut factory_opts = BlockBasedOptions::default();
        factory_opts.disable_cache();

        let mut opt = Options::default();
        opt.set_disable_auto_compactions(true);
        opt.create_if_missing(true);
        opt.set_block_based_table_factory(&factory_opts);

        Self {
            db: RocksdbSubstateStore::with_options(&opt, path),
            read_metrics: RefCell::new(BTreeMap::new()),
        }
    }
}

impl SubstateStoreWithMetrics<InMemorySubstateDatabase> {
    pub fn new_inmem() -> Self {
        Self {
            db: InMemorySubstateDatabase::standard(),
            read_metrics: RefCell::new(BTreeMap::new()),
        }
    }
}

impl<S: SubstateDatabase + CommittableSubstateDatabase> SubstateStoreWithMetrics<S> {
    pub fn calculate_median_points(&self) -> (Vec<(i32,i32)>, Vec<(i32,i32)>) {
        // 1. calculate max values
        let mut max_values = Vec::with_capacity(100000);
        let binding = self.read_metrics.borrow();
        for (_k, v) in binding.iter() {
            max_values.push(v.iter().max().unwrap());
        }

        // 2. filter out spikes and calculate medians
        //let peak_diff_division = 10;
        //let mut idx = 0;
        let mut data = Vec::with_capacity(100000);
        let mut median_data = Vec::new();
        for (k, v) in self.read_metrics.borrow().iter() {
            let mut w = v.iter().map(|i| *i).collect();
            let median = Self::calculate_median(&mut w);
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

        (data, median_data)
    }

    pub fn export_graph_and_print_summary(&mut self, data: &Vec<(i32, i32)>, median_data: &Vec<(i32, i32)>, output_png_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        // calculate axis max/min values
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
        let root = BitMapBackend::new(output_png_file, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;
        root.margin(20, 20, 20, 20);

        let mut scatter_ctx = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .margin(20)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
        scatter_ctx
            .configure_mesh()
            .x_desc("Size [bytes]")
            .y_desc("DB read duration [nanoseconds]")
            .axis_desc_style(("sans-serif", 16))
            .draw()?;
        // 1. draw all read points
        scatter_ctx
            .draw_series(
                data.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?
            .label(format!("Reads (count: {})", data.len()))
            .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
        // 2. draw median for each read series (basaed on same size)
        scatter_ctx
            .draw_series(
                median_data
                    .iter()
                    .map(|(x, y)| Cross::new((*x, *y), 6, RED)),
            )?
            .label("Median for each series")
            .legend(|(x, y)| Cross::new((x + 10, y), 6, RED));
        // 3. draw linear approximetion line
        scatter_ctx
            .draw_series(LineSeries::new(
                lin_x_axis
                    .values()
                    .map(|x| (x, (lin_slope * x as f64 + lin_intercept) as i32)),
                &BLUE,
            ))?
            .label(format!(
                "Linear approx.: f(x)={:.4}*x+{:.1}",
                lin_slope, lin_intercept
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        scatter_ctx
            .configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .label_font(("sans-serif", 16))
            .position(SeriesLabelPosition::UpperMiddle)
            .draw()?;

        root.present().expect("Unable to write result to file");

        // print some informations
        println!("Read count: {}", data.len());
        println!(
            "Distinct size read count: {}",
            self.read_metrics.borrow().len()
        );
        println!(
            "Read counts list (size, count): {:?}",
            self.read_metrics
                .borrow()
                .iter()
                .map(|(k, v)| (*k, v.len()))
                .collect::<Vec<(usize, usize)>>()
        );
        println!("Median points list (size, time): {:?}", median_data);
        println!(
            "Linear approx.:  f(size) = {} * size + {}\n",
            lin_slope, lin_intercept
        );

        Ok(())
    }


    fn calculate_median(data: &mut Vec<Duration>) -> Duration {
        data.sort();
        let center_idx = data.len() / 2;
        let median = data[center_idx];
        median
    }

    #[allow(dead_code)]
    fn discard_spikes(data: &mut Vec<Duration>, delta_range: Duration) {
        // 1. calculate median
        let median = Self::calculate_median(data);

        // 2. discard items out of median + range
        data.retain(|&i| {
            if i > median {
                i - median <= delta_range
            } else {
                median - i <= delta_range
            }
        });
    }

}

impl<S: SubstateDatabase + CommittableSubstateDatabase> SubstateDatabase for SubstateStoreWithMetrics<S> {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let start = std::time::Instant::now();
        let ret = self.db.get_substate(partition_key, sort_key);
        let duration = start.elapsed();

        if let Some(value) = ret {
            let exists = self.read_metrics.borrow().get(&value.len()).is_some();
            if exists {
                self.read_metrics
                    .borrow_mut()
                    .get_mut(&value.len())
                    .unwrap()
                    .push(duration);
            } else {
                self.read_metrics
                    .borrow_mut()
                    .insert(value.len(), vec![duration]);
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

impl<S: SubstateDatabase + CommittableSubstateDatabase> CommittableSubstateDatabase for SubstateStoreWithMetrics<S> {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        self.db.commit(database_updates)
    }
}

