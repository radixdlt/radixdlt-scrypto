use radix_engine_store_interface::interface::{
    CommittableSubstateDatabase, DatabaseUpdates, DbPartitionKey, DbSortKey, DbSubstateValue,
    PartitionEntry, SubstateDatabase, PartitionUpdates, DatabaseUpdate,
};
use radix_engine_stores::{rocks_db::{BlockBasedOptions, Options, /*LogLevel,*/ RocksdbSubstateStore}, memory_db::InMemorySubstateDatabase};
use std::{path::PathBuf, time::Duration, cell::RefCell, collections::BTreeMap};
use linreg::linear_regression_of;
use plotters::prelude::*;
use ::polyfit_rs::*;
use blake2::digest::{consts::U32, Digest};
use blake2::Blake2b;
use std::io::{self, Write};
use rand::Rng;

pub struct SubstateStoreWithMetrics<S>
where
    S: SubstateDatabase + CommittableSubstateDatabase
{
    db: S,
    read_metrics: RefCell<BTreeMap<usize, Vec<Duration>>>,
}

impl SubstateStoreWithMetrics<RocksdbSubstateStore> {
    pub fn bb(&self) -> Result<(), Box<dyn std::error::Error>> {
        let x_min = -5000;
        let x_max = 705000;
        let y_min = -1000;
        let y_max = 60000;

        let v = vec![(3, 1598), (4, 1322), (6, 691), (7, 1124), (10, 1568), (14, 1127), (21, 820), (31, 1654), (34, 3224), (39, 831), (55, 2343), (56, 4322), (61, 2041), (62, 1190), (65, 2414), (72, 1116), (79, 2184), (83, 1196), (92, 872), (93, 1929), (127, 3550), (454, 1316), (498, 1189), (797, 3796), (1825, 2464), (1950, 5780), (2711, 1311), (3203, 1637), (3931, 1063), (5269, 1519), (10007, 2355), (20008, 3100), (24161, 3251), (30008, 5202), (50008, 7777), (60008, 8807), (70008, 15102), (80008, 8093), (90008, 19500), (100008, 13218), (150008, 13713), (200008, 35152), (214772, 16167), (250008, 39320), (289696, 33243), (300008, 33737), (350008, 28390), (400008, 36344), (450008, 32362), (500008, 32423), (550008, 40388), (566509, 44225), (600008, 41552), (650008, 46570), (700008, 44102)];

        let v1 = vec![(3, 1598), (4, 1322), (6, 691), (7, 1124), (10, 1568), (14, 1127), (21, 820), (31, 1654), (34, 3224), (39, 831), (55, 2343), (56, 4322), (61, 2041), (62, 1190), (65, 2414), (72, 1116), (79, 2184), (83, 1196), (92, 872), (93, 1929), (127, 3550), (454, 1316), (498, 1189), (797, 3796),
            (1825, 2464), (1950, 5780), (2711, 1311), (3203, 1637), (3931, 1063), (5269, 1519),
            (10007, 2355), (20008, 3100), (24161, 3251)];
        let v3 = vec![ (30008, 5202), (50008, 7777), (60008, 8807), (70008, 15102), (80008, 8093), (90008, 19500)];
        let v4 = vec![(100008, 13218), (150008, 13713), (200008, 35152), (214772, 16167), (250008, 39320), (289696, 33243), (300008, 33737), (350008, 28390), (400008, 36344), (450008, 32362), (500008, 32423), (550008, 40388), (566509, 44225), (600008, 41552), (650008, 46570), (700008, 44102)];

        let (a1, b1): (f64, f64) = linear_regression_of(&v1).unwrap();
        let (a3, b3): (f64, f64) = linear_regression_of(&v3).unwrap();
        let (a4, b4): (f64, f64) = linear_regression_of(&v4).unwrap();
        let lin_x_axis1 = (0..25000).step(10);
        let lin_x_axis3 = (25000..100000).step(10);
        let lin_x_axis4 = (100000..x_max).step(10);

        let w_x: Vec<f64> = v.iter().map(|(a,_)|*a as f64).collect();
        let w_y: Vec<f64> = v.iter().map(|(_,b)|*b as f64).collect();
        let w = polyfit_rs::polyfit(w_x.as_slice(), w_y.as_slice(), 3).unwrap();
        let w_axis = (x_min..x_max).step(10);


        let root = BitMapBackend::new("/tmp/aa.png", (1024, 768)).into_drawing_area();
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
                v1.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?;
        scatter_ctx
            .draw_series(
                v3.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?;
        scatter_ctx
            .draw_series(
                v4.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?
            .label("Diff points (RocksDB - InMemory)")
            .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
        // 3. draw linear approximetion line
        scatter_ctx
            .draw_series(LineSeries::new(
                lin_x_axis1
                    .values()
                    .map(|x| (x, (a1 * x as f64 + b1) as i32 )),
                &BLUE,
            ))?
            .label(format!(
                "Linear approx. spline 1: f(x)={:.4}*x+{:.1}",
                a1, b1
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        scatter_ctx
            .draw_series(LineSeries::new(
                lin_x_axis3
                    .values()
                    .map(|x| (x, (a3 * x as f64 + b3) as i32 )),
                &BLUE,
            ))?
            .label(format!(
                "Linear approx. spline 2: f(x)={:.4}*x+{:.1}",
                a3, b3
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        scatter_ctx
            .draw_series(LineSeries::new(
                lin_x_axis4
                    .values()
                    .map(|x| (x, (a4 * x as f64 + b4) as i32 )),
                &BLUE,
            ))?
            .label(format!(
                "Linear approx. spline 3: f(x)={:.4}*x+{:.1}",
                a4, b4
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        // polynominal
        scatter_ctx
            .draw_series(LineSeries::new(
                w_axis
                    .values()
                    .map(|x| (x, (w[3] * x as f64 * x as f64 * x as f64 + w[2] * x as f64 * x as f64 + w[1] * x as f64 + w[0] ) as i32 )),
                &BLACK,
            ))?
            .label(format!(
                "3rd degree polynomial: {} {} {} {}",
                w[3], w[2], w[1], w[0]
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));
    
        scatter_ctx
            .configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .label_font(("sans-serif", 16))
            .position(SeriesLabelPosition::UpperMiddle)
            .draw()?;

        root.present().expect("Unable to write result to file");
        Ok(())
    }

    pub fn aa(&self) -> Result<(), Box<dyn std::error::Error>> {
        let x_min = -5000;
        let x_max = 705000;
        let y_min = -1000;
        let y_max = 60000;

        let v = vec![(3, 1598), (4, 1322), (6, 691), (7, 1124), (10, 1568), (14, 1127), (21, 820), (31, 1654), (34, 3224), (39, 831), (55, 2343), (56, 4322), (61, 2041), (62, 1190), (65, 2414), (72, 1116), (79, 2184), (83, 1196), (92, 872), (93, 1929), (127, 3550), (454, 1316), (498, 1189), (797, 3796), (1825, 2464), (1950, 5780), (2711, 1311), (3203, 1637), (3931, 1063), (5269, 1519), (10007, 2355), (20008, 3100), (24161, 3251), (30008, 5202), (50008, 7777), (60008, 8807), (70008, 15102), (80008, 8093), (90008, 19500), (100008, 13218), (150008, 13713), (200008, 35152), (214772, 16167), (250008, 39320), (289696, 33243), (300008, 33737), (350008, 28390), (400008, 36344), (450008, 32362), (500008, 32423), (550008, 40388), (566509, 44225), (600008, 41552), (650008, 46570), (700008, 44102)];

        let v1 = vec![(3, 1598), (4, 1322), (6, 691), (7, 1124), (10, 1568), (14, 1127), (21, 820), (31, 1654), (34, 3224), (39, 831), (55, 2343), (56, 4322), (61, 2041), (62, 1190), (65, 2414), (72, 1116), (79, 2184), (83, 1196), (92, 872), (93, 1929), (127, 3550), (454, 1316), (498, 1189), (797, 3796),
            (1825, 2464), (1950, 5780), (2711, 1311), (3203, 1637), (3931, 1063), (5269, 1519),
            (10007, 2355), (20008, 3100), (24161, 3251)];
        let v3 = vec![ (30008, 5202), (50008, 7777), (60008, 8807), (70008, 15102), (80008, 8093), (90008, 19500)];
        let v4 = vec![(100008, 13218), (150008, 13713), (200008, 35152), (214772, 16167), (250008, 39320), (289696, 33243), (300008, 33737), (350008, 28390), (400008, 36344), (450008, 32362), (500008, 32423), (550008, 40388), (566509, 44225), (600008, 41552), (650008, 46570), (700008, 44102)];

        let (a1, b1): (f64, f64) = linear_regression_of(&v1).unwrap();
        let (a3, b3): (f64, f64) = linear_regression_of(&v3).unwrap();
        let (a4, b4): (f64, f64) = linear_regression_of(&v4).unwrap();
        let lin_x_axis1 = (0..25000).step(10);
        let lin_x_axis3 = (25000..100000).step(10);
        let lin_x_axis4 = (100000..x_max).step(10);

        let w_x: Vec<f64> = v.iter().map(|(a,_)|*a as f64).collect();
        let w_y: Vec<f64> = v.iter().map(|(_,b)|*b as f64).collect();
        let w = polyfit_rs::polyfit(w_x.as_slice(), w_y.as_slice(), 3).unwrap();
        let w_axis = (x_min..x_max).step(10);


        let root = BitMapBackend::new("/tmp/aa.png", (1024, 768)).into_drawing_area();
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
                v1.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?;
        scatter_ctx
            .draw_series(
                v3.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?;
        scatter_ctx
            .draw_series(
                v4.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?
            .label("Diff points (RocksDB - InMemory)")
            .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
        // 3. draw linear approximetion line
        scatter_ctx
            .draw_series(LineSeries::new(
                lin_x_axis1
                    .values()
                    .map(|x| (x, (a1 * x as f64 + b1) as i32 )),
                &BLUE,
            ))?
            .label(format!(
                "Linear approx. spline 1: f(x)={:.4}*x+{:.1}",
                a1, b1
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        scatter_ctx
            .draw_series(LineSeries::new(
                lin_x_axis3
                    .values()
                    .map(|x| (x, (a3 * x as f64 + b3) as i32 )),
                &BLUE,
            ))?
            .label(format!(
                "Linear approx. spline 2: f(x)={:.4}*x+{:.1}",
                a3, b3
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        scatter_ctx
            .draw_series(LineSeries::new(
                lin_x_axis4
                    .values()
                    .map(|x| (x, (a4 * x as f64 + b4) as i32 )),
                &BLUE,
            ))?
            .label(format!(
                "Linear approx. spline 3: f(x)={:.4}*x+{:.1}",
                a4, b4
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        // polynominal
        scatter_ctx
            .draw_series(LineSeries::new(
                w_axis
                    .values()
                    .map(|x| (x, (w[3] * x as f64 * x as f64 * x as f64 + w[2] * x as f64 * x as f64 + w[1] * x as f64 + w[0] ) as i32 )),
                &BLACK,
            ))?
            .label(format!(
                "3rd degree polynomial: {} {} {} {}",
                w[3], w[2], w[1], w[0]
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));
    
        scatter_ctx
            .configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .label_font(("sans-serif", 16))
            .position(SeriesLabelPosition::UpperMiddle)
            .draw()?;

        root.present().expect("Unable to write result to file");
        Ok(())
    }
    pub fn new_rocksdb(path: PathBuf) -> Self {
        let mut factory_opts = BlockBasedOptions::default();
        factory_opts.disable_cache();

        let mut opt = Options::default();
        opt.set_disable_auto_compactions(true);
        opt.create_if_missing(true);
        opt.set_block_based_table_factory(&factory_opts);
        //opt.set_keep_log_file_num(1);
        //opt.set_log_level(LogLevel::Fatal);
        //opt.set_stats_dump_period_sec(0);
        //opt.set_target_file_size_base();
        //opt.set_target_file_size_multiplier(2);
        //opt.set_max_open_files(4000);

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

    pub fn drop_edge_values(&mut self) {
        for (_, v) in self.read_metrics.borrow_mut().iter_mut() {
            v.sort();
            v.pop();
            v.remove(0);
        }
    }

    pub fn calculate_percent_to_max_points(&mut self, percent: f32) -> Vec<(i32,i32)> {
        assert!(percent <= 100f32);
        let mut output_values = Vec::new();
        let mut binding = self.read_metrics.borrow_mut();
        for (k, v) in binding.iter_mut() {
            v.sort();
            let idx = (((v.len() - 1) as f32 * percent) / 100f32).round() as usize;
            output_values.push((*k as i32, v[idx].as_nanos() as i32));
        }
        output_values
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
        let (lin_slope, lin_intercept): (f64, f64) = linear_regression_of(&median_data).unwrap();
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

    pub fn export_graph_and_print_summary_for_two_series(&mut self, data_series1: &Vec<(i32, i32)>, data_series2: &Vec<(i32, i32)>, output_png_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        // calculate diff points
        assert_eq!(data_series1.len(), data_series2.len());
        let mut v1 = data_series1.clone();
        for (idx, (size, median_value)) in v1.iter_mut().enumerate() {
            assert_eq!(*size, data_series2[idx].0);
            *median_value -= data_series2[idx].1;

        }
        // calculate linear approximation of diff points
        let (lin_slope, lin_intercept): (f64, f64) = linear_regression_of(&v1).unwrap();

        // calculate axis max/min values
        let y_ofs = 1000;
        let x_ofs = 5000;
        let x_min = data_series1.iter().map(|i| i.0).min().unwrap() - x_ofs;
        let x_max = data_series1.iter().map(|i| i.0).max().unwrap() + x_ofs;
        let y_min = data_series1.iter().map(|i| i.1).min().unwrap() - y_ofs;
        let y_max = data_series1.iter().map(|i| i.1).max().unwrap() + y_ofs;

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
        // 1. draw read series1 points
        scatter_ctx
            .draw_series(
                data_series1.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )?
            .label("RocksDB read (median points)")
            .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
        // 2. draw read series2 points
        scatter_ctx
            .draw_series(
                data_series2.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, BLUE.filled())),
            )?
            .label("InMemory read (median points)")
            .legend(|(x, y)| Circle::new((x + 10, y), 2, BLUE.filled()));
        // 3. draw read series1-series2 points
        scatter_ctx
            .draw_series(
                v1.iter()
                    .map(|(x, y)| Cross::new((*x, *y), 6, MAGENTA)),
            )?
            .label("Diff points (RocksDB/green - InMemory/blue)")
            .legend(|(x, y)| Cross::new((x + 10, y), 6, MAGENTA));
        // 4. draw linear approximetion line
        scatter_ctx
            .draw_series(LineSeries::new(
                lin_x_axis
                    .values()
                    .map(|x| (x, (lin_slope * x as f64 + lin_intercept) as i32)),
                &RED,
            ))?
            .label(format!(
                "Linear approx. of diff points: f(x)={:.4}*x+{:.1}",
                lin_slope, lin_intercept
            ))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
        scatter_ctx
            .configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .label_font(("sans-serif", 16))
            .position(SeriesLabelPosition::UpperMiddle)
            .draw()?;

        root.present().expect("Unable to write result to file");

        // print some informations
        println!("Points list (size, time): {:?}", v1);
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


#[test]
fn test_store_db() {

    // RocksDB part
    let path = PathBuf::from(r"/tmp/radix-scrypto-db");
    // clean database
    std::fs::remove_dir_all(path.clone()).ok();

    let min_size = 1;
    let max_size = 4 * 1024 * 1024;
    let size_step = 10 * 1024;
    let count = 10usize;
    let read_repeats = 200usize;

    let mut data_index_vector: Vec<(DbPartitionKey, DbSortKey, usize)> = Vec::with_capacity(max_size);

    {
        println!("Preparing database...");
        let mut p_key_cnt = 1u32;
        let mut substate_db = SubstateStoreWithMetrics::new_rocksdb(path.clone());
        let mut sort_key_value: usize = 0;
        for size in (min_size..=max_size).step_by(size_step) {
            let mut input_data = DatabaseUpdates::new();
            for _ in 0..count {
                let value = DatabaseUpdate::Set(vec![1; size]);

                let plain_bytes = sort_key_value.to_be_bytes().to_vec();
                let mut hashed_prefix: Vec<u8> = Blake2b::<U32>::digest(plain_bytes.clone()).to_vec();
                hashed_prefix.extend(plain_bytes);

                let sort_key = DbSortKey(hashed_prefix);
                sort_key_value += 1;

                let mut partition = PartitionUpdates::new();
                partition.insert(sort_key.clone(), value);

                let partition_key = DbPartitionKey(p_key_cnt.to_be_bytes().to_vec());
                p_key_cnt += 1;

                data_index_vector.push( (partition_key.clone(), sort_key, size) );

                input_data.insert(partition_key, partition);
            }
            substate_db.commit(&input_data);
        }
        println!("  done\n");
    }

    println!("Random read start...");

    // reopen database
    let mut substate_db = SubstateStoreWithMetrics::new_rocksdb(path);

    let mut rng = rand::thread_rng();

    //let mut p_key_cnt = 1u32;
    for i in 0..read_repeats {
        let time_start = std::time::Instant::now();
        let mut idx_vector: Vec<usize> = (0..data_index_vector.len()).collect();

        for j in 0..data_index_vector.len() {
            assert!(!idx_vector.is_empty());
            let idx = rng.gen_range(0..idx_vector.len());

            let (p,s,v) = &data_index_vector[idx_vector[idx]];

            print!("\rRead {}/{}", j + 1, data_index_vector.len());
            std::io::stdout().flush().ok();
    
            let read_value = substate_db.get_substate(&p, &s);
    
            assert!(read_value.is_some());
            assert_eq!(read_value.unwrap().len(), *v);

            idx_vector.remove(idx);
        }

        // sequential read
        // for (j, (p,s,v)) in data_index_vector.iter().enumerate() {
        //     print!("\rRead {}/{}", j + 1, data_index_vector.len());
        //     std::io::stdout().flush().ok();

        //     let read_value = substate_db.get_substate(&p, &s);

        //     assert!(read_value.is_some());
        //     assert_eq!(read_value.unwrap().len(), *v);
        // }

        let time_end = std::time::Instant::now();
        let mut duration = time_end.checked_duration_since(time_start).unwrap().as_secs();
        if duration == 0 {
            duration = 1;
        }
        println!("\rRound {}/{}  read time: {} s, left: {} s\r", i + 1, read_repeats, duration, (read_repeats - (i + 1)) * duration as usize );
    }

    println!("Read done");


    substate_db.drop_edge_values();
    let output_data = substate_db.calculate_percent_to_max_points(95f32);

    let mut data = Vec::with_capacity(100000);
    for (k, v) in substate_db.read_metrics.borrow().iter() {
        for i in v {
            data.push((*k as i32, i.as_nanos() as i32));
        }
    }

    // export results
    substate_db.export_graph_and_print_summary(&data, &output_data, "/tmp/aa_1.png").unwrap();

}