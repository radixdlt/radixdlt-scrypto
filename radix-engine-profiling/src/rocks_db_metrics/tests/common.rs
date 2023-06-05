use super::super::*;
use linreg::linear_regression_of;
use plotters::prelude::*;
use radix_engine_store_interface::{
    db_key_mapper::*,
    interface::{
        CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates, DbPartitionKey,
        DbSortKey, PartitionUpdates, SubstateDatabase,
    },
};
use rand::{Rng, rngs::ThreadRng};
#[allow(unused_imports)]
use std::{io::Write, path::PathBuf};



pub fn drop_highest_and_lowest_value<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_store: &SubstateStoreWithMetrics<S>,
    count: usize,
) {
    if substate_store.read_metrics.borrow().len() > 2 * count {
        for (_, v) in substate_store.read_metrics.borrow_mut().iter_mut() {
            v.sort();
            for _ in 0..count {
                v.pop();
                v.remove(0);
            }
        }
    }
    if substate_store.commit_metrics.borrow().len() > 2 * count {
        for (_, v) in substate_store.commit_metrics.borrow_mut().iter_mut() {
            v.sort();
            for _ in 0..count {
                v.pop();
                v.remove(0);
            }
        }
    }
}

pub fn calculate_percent_to_max_points(
    data: &mut BTreeMap<usize, Vec<Duration>>,
    percent: f32,
) -> Vec<(f32, f32)> {
    assert!(percent <= 100f32);
    let mut output_values = Vec::new();
    for (k, v) in data.iter_mut() {
        v.sort();
        let idx = (((v.len() - 1) as f32 * percent) / 100f32).round() as usize;
        output_values.push((*k as f32, v[idx].as_micros() as f32));
    }
    output_values
}

pub fn generate_commit_data(rng: &mut ThreadRng, value_size: usize) -> (DbPartitionKey, DbSortKey, IndexMap<DbSortKey, DatabaseUpdate>) {
    let mut value_data: DbSubstateValue = vec![0u8; value_size];
    rng.fill(value_data.as_mut_slice());
    let value = DatabaseUpdate::Set(value_data);

    let mut node_id_value = [0u8; NodeId::UUID_LENGTH];
    rng.fill(&mut node_id_value);
    let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);
    let partition_key =
        SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(0u8));

    let mut substate_key_value = [0u8; NodeId::LENGTH];
    rng.fill(&mut substate_key_value);
    let sort_key = SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(
        substate_key_value.into(),
    ));

    let mut partition = PartitionUpdates::new();
    partition.insert(sort_key.clone(), value);

    (partition_key, sort_key, partition)
}

pub fn prepare_db<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_db: &mut S,
    min_size: usize,
    max_size: usize,
    step: usize,
    writes_count: usize,
    random_size: bool,
) -> Vec<(DbPartitionKey, DbSortKey, usize)> {
    let mut data_index_vector: Vec<(DbPartitionKey, DbSortKey, usize)> =
        Vec::with_capacity(max_size);

    print!(
        "Preparing database ({}, {}, {}, {})...",
        min_size, max_size, step, writes_count
    );
    std::io::stdout().flush().ok();
    let mut rng = rand::thread_rng();

    if random_size {
        println!("");
        let batch_size = writes_count / 100;
        for i in 0..writes_count / batch_size {
            let mut input_data = DatabaseUpdates::with_capacity(batch_size);
            for _ in 0..batch_size {
                print!("\rRound {}/{}", i + 1, writes_count / batch_size );
                std::io::stdout().flush().ok();

                let size = rng.gen_range(min_size..=max_size);

                let (partition_key, sort_key, partition) = generate_commit_data(&mut rng, size);

                data_index_vector.push((partition_key.clone(), sort_key, size));

                input_data.insert(partition_key, partition);
            }
            substate_db.commit(&input_data);
        }
    } else {
        for size in (min_size..=max_size).step_by(step) {
            let mut input_data = DatabaseUpdates::new();
            for _ in 0..writes_count {
                let (partition_key, sort_key, partition) = generate_commit_data(&mut rng, size);

                data_index_vector.push((partition_key.clone(), sort_key, size));

                input_data.insert(partition_key, partition);
            }
            substate_db.commit(&input_data);
        }
    }
    println!("  done");

    data_index_vector
}

#[allow(dead_code)]
pub fn export_one_graph(
    caption: &str,
    data: &Vec<(f32, f32)>,
    output_png_file: &str,
    original_data: &RefCell<BTreeMap<usize, Vec<Duration>>>,
    y_max_value: Option<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    // calculate axis max/min values
    let x_min = data.iter().map(|i| (i.0 as i32)).min().unwrap() as f32;
    let x_max = data.iter().map(|i| (i.0 as i32)).max().unwrap() as f32;
    let y_min = data.iter().map(|i| i.1 as i32).min().unwrap() as f32;
    let y_max: f32 = if let Some(y) = y_max_value {
        y
    } else {
        data.iter().map(|i| i.1 as i32).max().unwrap() as f32
    };

    // draw scatter plot
    let root = BitMapBackend::new(output_png_file, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?; 
    root.margin(20, 20, 20, 20);

    let mut scatter_ctx = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 20).into_font())
        .x_label_area_size(40)
        .y_label_area_size(80)
        .margin(20)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    scatter_ctx
        .configure_mesh()
        .x_desc("Size [bytes]")
        .y_desc("Duration [microseconds]")
        .axis_desc_style(("sans-serif", 16))
        .draw()?;
    // 1. draw all points
    scatter_ctx
        .draw_series(
            data.iter()
                .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
        )?
        .label(format!("Points (count: {})", data.len()))
        .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
    scatter_ctx
        .configure_series_labels()
        .background_style(&WHITE)
        .border_style(&BLACK)
        .label_font(("sans-serif", 16))
        .position(SeriesLabelPosition::UpperMiddle)
        .draw()?;

    root.present().expect("Unable to write result to file");

    // print some informations
    println!("Points count: {}", data.len());
    println!(
        "Distinct size point count: {}",
        original_data.borrow().len()
    );
    println!(
        "Points counts list (size, count): {:?}",
        original_data
            .borrow()
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect::<Vec<(usize, usize)>>()
    );
    println!(
        "Points list (size, time[µs]): {:?}",
        data
    );

    Ok(())
}

pub fn calculate_axis_ranges(data: &Vec<(f32, f32)>, x_ofs: Option<f32>, y_ofs: Option<f32>) -> (f32, f32, f32, f32) {
    let x_ofs = x_ofs.unwrap_or_else(|| 0f32);
    let y_ofs = y_ofs.unwrap_or_else(|| 0f32);
    let x_min = data.iter().map(|i| (i.0 as i32)).min().unwrap() as f32 - x_ofs;
    let x_max = data.iter().map(|i| (i.0 as i32)).max().unwrap() as f32 + x_ofs;
    let y_min = data.iter().map(|i| i.1 as i32).min().unwrap() as f32 - y_ofs;
    let y_max = data.iter().map(|i| i.1 as i32).max().unwrap() as f32 + y_ofs;
    (x_min, x_max, y_min, y_max)
}

pub fn export_graph_and_print_summary(
    caption: &str,
    data: &Vec<(f32, f32)>,
    output_data: &Vec<(f32, f32)>,
    output_png_file: &str,
    output_data_name: &str,
    original_data: &BTreeMap<usize, Vec<Duration>>,
    axis_ranges: (f32, f32, f32, f32),
    x_axis_description: Option<&str>
) -> Result<(), Box<dyn std::error::Error>> {
    // calculate axis max/min values
    let x_min = axis_ranges.0;
    let x_max = axis_ranges.1;
    let mut y_min = axis_ranges.2;
    let y_max = axis_ranges.3;

    // 4. calculate linear approximation
    let (lin_slope, lin_intercept): (f64, f64) = linear_regression_of(&output_data).unwrap();
    let lin_x_axis = (x_min as f32..(x_max + 1f32) as f32).step(1f32);
    if lin_intercept < y_min.into() {
        y_min = lin_intercept as f32;
    }

    // draw scatter plot
    let root = BitMapBackend::new(output_png_file, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;
    root.margin(20, 20, 20, 20);

    let mut scatter_ctx = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 20).into_font())
        .x_label_area_size(40)
        .y_label_area_size(80)
        .margin(20)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    scatter_ctx
        .configure_mesh()
        .x_desc(x_axis_description.unwrap_or_else(|| "Size [bytes]"))
        .y_desc("Duration [microseconds]")
        .axis_desc_style(("sans-serif", 16))
        .draw()?;
    // 1. draw all read points
    scatter_ctx
        .draw_series(
            data.iter()
                .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
        )?
        .label(format!("Points (count: {})", data.len()))
        .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
    // 2. draw median for each read series (basaed on same size)
    scatter_ctx
        .draw_series(
            output_data
                .iter()
                .map(|(x, y)| Cross::new((*x, *y), 6, RED)),
        )?
        .label(output_data_name)
        .legend(|(x, y)| Cross::new((x + 10, y), 6, RED));
    // 3. draw linear approximetion line
    scatter_ctx
        .draw_series(LineSeries::new(
            lin_x_axis
                .values()
                .map(|x| (x, (lin_slope * x as f64 + lin_intercept) as f32)),
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
    println!("Points count: {}", data.len());
    println!(
        "Distinct size point count: {}",
        original_data.len()
    );
    println!(
        "Points counts list (size, count): {:?}",
        original_data
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect::<Vec<(usize, usize)>>()
    );
    println!(
        "{} points list (size, time[µs]): {:?}",
        output_data_name, output_data
    );
    println!(
        "Linear approx.:  f(size) = {} * size + {}\n",
        lin_slope, lin_intercept
    );
    println!("Output graph file: {}\n", output_png_file);

    Ok(())
}



pub fn export_graph_and_print_summary_for_two_series(
    caption: &str,
    data_series1: &Vec<(f32, f32)>,
    data_series2: &Vec<(f32, f32)>,
    output_png_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // calculate diff points
    assert_eq!(data_series1.len(), data_series2.len());
    let mut v1 = data_series1.clone();
    for (idx, (size, diff_value)) in v1.iter_mut().enumerate() {
        assert_eq!(*size, data_series2[idx].0);
        *diff_value -= data_series2[idx].1;
    }
    // calculate linear approximation of diff points
    let (lin_slope, lin_intercept): (f64, f64) = linear_regression_of(&v1).unwrap();

    // calculate linethrough 1st and last diff points
    let v2: Vec<(f32, f32)> = vec![
        *data_series1.first().unwrap(),
        *data_series1.last().unwrap(),
    ];
    let (lin_slope_2, lin_intercept_2): (f64, f64) = linear_regression_of(&v2).unwrap();

    // calculate axis max/min values
    let y_ofs = 10f32;
    let x_ofs = 5000f32;
    let x_min = -x_ofs;
    let x_max = data_series1.iter().map(|i| i.0 as i32).max().unwrap() as f32 + x_ofs;
    let y_min = 0f32;
    let y_max = data_series1.iter().map(|i| i.1 as i32).max().unwrap() as f32 + y_ofs;

    let lin_x_axis = (x_min..x_max).step(10f32);

    // draw scatter plot
    let root = BitMapBackend::new(output_png_file, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;
    root.margin(20, 20, 20, 20);

    let mut scatter_ctx = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 20).into_font())
        .x_label_area_size(40)
        .y_label_area_size(80)
        .margin(20)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    scatter_ctx
        .configure_mesh()
        .x_desc("Size [bytes]")
        .y_desc("DB read duration [microseconds]")
        .axis_desc_style(("sans-serif", 16))
        .draw()?;
    // 1. draw read series1 points
    scatter_ctx
        .draw_series(
            data_series1
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
        )?
        .label("RocksDB read (95th percentile)")
        .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
    // 2. draw read series2 points
    scatter_ctx
        .draw_series(
            data_series2
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 2, BLUE.filled())),
        )?
        .label("InMemory read (95th percentile)")
        .legend(|(x, y)| Circle::new((x + 10, y), 2, BLUE.filled()));
    // 3. draw read series1-series2 points
    scatter_ctx
        .draw_series(v1.iter().map(|(x, y)| Cross::new((*x, *y), 6, MAGENTA)))?
        .label("Diff points (RocksDB/green - InMemory/blue)")
        .legend(|(x, y)| Cross::new((x + 10, y), 6, MAGENTA));
    // 4. draw linear approximetion line
    scatter_ctx
        .draw_series(LineSeries::new(
            lin_x_axis
                .values()
                .map(|x| (x, (lin_slope * x as f64 + lin_intercept) as f32)),
            &RED,
        ))?
        .label(format!(
            "Linear approx. of diff points: f(x)={:.4}*x+{:.1}",
            lin_slope, lin_intercept
        ))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    scatter_ctx
        .draw_series(LineSeries::new(
            lin_x_axis
                .values()
                .map(|x| (x, (lin_slope_2 * x as f64 + lin_intercept_2) as f32)),
            &BLACK,
        ))?
        .label(format!(
            "Line by 1st and last RocksDB point: f(x)={:.4}*x+{:.1}",
            lin_slope_2, lin_intercept_2
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

    // print some informations
    println!("Points list (size, time[µs]): {:?}", v1);
    println!(
        "Linear approx.:  f(size) = {} * size + {}\n",
        lin_slope, lin_intercept
    );
    println!(
        "Liny by 1st and last RocksDB point:  f(size) = {} * size + {}\n",
        lin_slope_2, lin_intercept_2
    );

    Ok(())
}


pub fn print_read_not_found_results<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_store: &SubstateStoreWithMetrics<S>,
) {
    let v = &substate_store.read_not_found_metrics.borrow();
    let min = v.iter().min().unwrap().as_nanos();
    let max = v.iter().max().unwrap().as_nanos();
    let avg = v.iter().sum::<Duration>().as_nanos() as usize / v.len();
    println!(
        "Read not found times [ns]: min={} max={} avg={}\n",
        min, max, avg
    );
}
