use super::super::*;
use linreg::linear_regression_of;
use plotters::prelude::*;
use radix_substate_store_interface::db_key_mapper::*;
use rand::{rngs::ThreadRng, Rng};
#[allow(unused_imports)]
use std::{io::Write, path::PathBuf};

pub const SUBSTATE_KEY_LENGTH: usize = 30;

macro_rules! number_to_str {
    ( $x:expr, $precission:expr ) => {{
        if $x < 0f32 {
            format!("{:.1$}", $x, $precission)
        } else {
            format!("+{:.1$}", $x, $precission)
        }
    }};
}

pub fn generate_range(min: usize, max: usize, values_count: usize) -> Vec<usize> {
    let mut v = Vec::new();

    assert!(min <= max);
    assert!(values_count > 0);

    let n = values_count as f32;
    let step = 1f32 / n;
    let threshold = 0.9f32; // in range 0..1

    const S: f32 = 9f32;
    const T: f32 = 14f32;
    const P: i32 = 4;

    let point_a = (threshold, S + (T * threshold).powi(P));
    let point_b = (1f32, max as f32);
    let m = (point_a.1 - point_b.1) / (point_a.0 - point_b.0);

    let mut i = step;
    while i <= 1f32 {
        let value = if i < threshold {
            // f(x) = S + ( T * x )^P
            S + (T * i).powi(P)
        } else {
            // A=[threshold, f(threshold)] B=[1,max]
            // M=(Ay-By)/(Ax-Bx)
            // f(x) = x * M + (Ay - M * Ax)
            if i + step > 1f32 {
                i = 1f32;
            }
            i * m + (point_a.1 - m * point_a.0)
        };
        if value >= min as f32 {
            v.push(value.ceil() as usize);
        }
        i += step;
    }

    if v.len() < values_count {
        v.insert(0, min);
    } else {
        v[0] = min;
    }

    v
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

pub fn discard_spikes(data: &mut BTreeMap<usize, Vec<Duration>>, delta_range: f32) {
    for (_, v) in data.iter_mut() {
        v.sort();
        let center_idx = v.len() / 2;
        let median = v[center_idx].as_micros() as f32;

        v.retain(|&i| {
            let value = i.as_micros() as f32;
            if value > median {
                value - median <= delta_range
            } else {
                median - value <= delta_range
            }
        });
    }
}

pub fn generate_commit_data(
    partition: &mut IndexMap<DbSortKey, DatabaseUpdate>,
    rng: &mut ThreadRng,
    value_size: usize,
) -> DbSortKey {
    let mut value_data: DbSubstateValue = vec![0u8; value_size];
    rng.fill(value_data.as_mut_slice());
    let value = DatabaseUpdate::Set(value_data);

    let mut substate_key_value = [0u8; SUBSTATE_KEY_LENGTH];
    rng.fill(&mut substate_key_value);
    let sort_key =
        SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(substate_key_value.into()));

    partition.insert(sort_key.clone(), value);

    sort_key
}

pub fn prepare_db<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_db: &mut S,
    min_size: usize,
    max_size: usize,
    values_count: usize,
    writes_count: usize,
) -> Vec<(DbPartitionKey, DbSortKey, usize)> {
    let mut data_index_vector: Vec<(DbPartitionKey, DbSortKey, usize)> =
        Vec::with_capacity(max_size);

    println!(
        "Preparing database ({}, {}, {}, {})...",
        min_size, max_size, values_count, writes_count
    );
    let mut rng = rand::thread_rng();

    let substate_size_list = generate_range(min_size, max_size, values_count);

    for i in 0..writes_count {
        print!("\rNode {}/{}  ", i + 1, writes_count);
        std::io::stdout().flush().ok();

        let mut input_data = index_map_new();

        let mut node_id_value = [0u8; NodeId::RID_LENGTH];
        rng.fill(&mut node_id_value);
        let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);
        let partition_key =
            SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(0u8));
        let mut partition = index_map_new();

        for size in substate_size_list.iter() {
            let sort_key = generate_commit_data(&mut partition, &mut rng, *size);

            data_index_vector.push((partition_key.clone(), sort_key, *size));
        }
        input_data.insert(partition_key.clone(), partition.clone());

        substate_db.commit(&DatabaseUpdates::from_delta_maps(input_data));
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
        .y_desc("Duration [µs]")
        .axis_desc_style(("sans-serif", 16))
        .draw()?;
    // Draw all points
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

    // print some information
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
    println!("Points list (size, time[µs]): {:?}", data);

    Ok(())
}

pub fn calculate_axis_ranges(
    data: &Vec<(f32, f32)>,
    x_ofs: Option<f32>,
    y_ofs: Option<f32>,
) -> (f32, f32, f32, f32) {
    let x_ofs = x_ofs.unwrap_or(0f32);
    let y_ofs = y_ofs.unwrap_or(0f32);
    let x_min = data.iter().map(|i| (i.0 as i32)).min().unwrap_or_default() as f32 - x_ofs;
    let x_max = data.iter().map(|i| (i.0 as i32)).max().unwrap_or_default() as f32 + x_ofs;
    let mut y_min = data.iter().map(|i| i.1 as i32).min().unwrap_or_default() as f32 - y_ofs;
    let y_max = data.iter().map(|i| i.1 as i32).max().unwrap_or_default() as f32 + y_ofs;
    if y_min > 0f32 {
        y_min = 0f32;
    }
    (x_min, x_max, y_min, y_max)
}

pub fn calculate_axis_ranges_for_two_series(
    data_series1: &Vec<(f32, f32)>,
    data_series2: &Vec<(f32, f32)>,
    x_ofs: Option<f32>,
    y_ofs: Option<f32>,
) -> (f32, f32, f32, f32) {
    let x_ofs = x_ofs.unwrap_or(0f32);
    let y_ofs = y_ofs.unwrap_or(0f32);
    let x_min1 = data_series1
        .iter()
        .map(|i| i.0 as i32)
        .min()
        .unwrap_or_default() as f32;
    let x_max1 = data_series1
        .iter()
        .map(|i| i.0 as i32)
        .max()
        .unwrap_or_default() as f32;
    let y_min1 = data_series1
        .iter()
        .map(|i| i.1 as i32)
        .min()
        .unwrap_or_default() as f32;
    let y_max1 = data_series1
        .iter()
        .map(|i| i.1 as i32)
        .max()
        .unwrap_or_default() as f32;
    let x_min2 = data_series2
        .iter()
        .map(|i| i.0 as i32)
        .min()
        .unwrap_or_default() as f32;
    let x_max2 = data_series2
        .iter()
        .map(|i| i.0 as i32)
        .max()
        .unwrap_or_default() as f32;
    let y_min2 = data_series2
        .iter()
        .map(|i| i.1 as i32)
        .min()
        .unwrap_or_default() as f32;
    let y_max2 = data_series2
        .iter()
        .map(|i| i.1 as i32)
        .max()
        .unwrap_or_default() as f32;
    let x_min = if x_min1 < x_min2 {
        x_min1 - x_ofs
    } else {
        x_min2 - x_ofs
    };
    let x_max = if x_max1 > x_max2 {
        x_max1 + x_ofs
    } else {
        x_max2 + x_ofs
    };
    let mut y_min = if y_min1 < y_min2 {
        y_min1 - y_ofs
    } else {
        y_min2 - y_ofs
    };
    let y_max = if y_max1 > y_max2 {
        y_max1 + y_ofs
    } else {
        y_max2 + y_ofs
    };
    if y_min > 0f32 {
        y_min = 0f32;
    }
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
    x_axis_description: Option<&str>,
    draw_zoom: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // calculate axis max/min values
    let x_min = axis_ranges.0;
    let mut x_max = axis_ranges.1;
    let mut y_min = axis_ranges.2;
    let mut y_max = axis_ranges.3;

    // calculate linear approximation
    let (lin_slope, lin_intercept): (f32, f32) =
        linear_regression_of(&output_data).unwrap_or_default();
    let lin_x_axis = (x_min as f32..(x_max + 1f32) as f32).step(1f32);
    if lin_intercept < y_min.into() {
        y_min = lin_intercept as f32;
    }
    y_max = find_y_max(lin_slope, lin_intercept, None, None, x_max, y_max) * 1.1f32;

    // draw scatter plot
    let root_area = BitMapBackend::new(output_png_file, (1280, 1024)).into_drawing_area();
    root_area.fill(&WHITE)?;
    root_area.margin(20, 20, 20, 20);
    let (upper, lower) = if draw_zoom {
        root_area.split_vertically(700)
    } else {
        root_area.split_vertically(1024)
    };

    let mut scatter_ctx = ChartBuilder::on(&upper)
        .caption(caption, ("sans-serif", 20).into_font())
        .x_label_area_size(40)
        .y_label_area_size(80)
        .margin(20)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    scatter_ctx
        .configure_mesh()
        .x_desc(x_axis_description.unwrap_or_else(|| "Size [bytes]"))
        .y_desc("Duration [µs]")
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
    // 3. draw linear approximation line
    scatter_ctx
        .draw_series(LineSeries::new(
            lin_x_axis
                .values()
                .map(|x| (x, lin_slope * x + lin_intercept)),
            &BLUE,
        ))?
        .label(format!(
            "Linear approx.: f(x)={:.3e}*x{}",
            lin_slope,
            number_to_str!(lin_intercept, 1)
        ))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    scatter_ctx
        .configure_series_labels()
        .background_style(&WHITE)
        .border_style(&BLACK)
        .label_font(("sans-serif", 16))
        .position(SeriesLabelPosition::UpperMiddle)
        .draw()?;

    if draw_zoom {
        // draw 2nd graph
        x_max /= 25f32;
        let y_max = output_data
            .iter()
            .filter(|(x, _y)| *x < x_max)
            .map(|(_x, y)| (*y as i32))
            .max()
            .unwrap() as f32
            * 2f32;

        let mut scatter_lower_ctx = ChartBuilder::on(&lower)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .margin(20)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
        scatter_lower_ctx
            .configure_mesh()
            .x_desc(x_axis_description.unwrap_or_else(|| "Size [bytes]"))
            .y_desc("Duration [µs]")
            .axis_desc_style(("sans-serif", 16))
            .y_max_light_lines(2)
            .draw()?;
        // 1. draw all read points
        scatter_lower_ctx.draw_series(
            data.iter()
                .filter(|(x, _y)| *x < x_max)
                .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
        )?;
        // 2. draw median for each read series (basaed on same size)
        scatter_lower_ctx.draw_series(
            output_data
                .iter()
                .filter(|(x, _y)| *x < x_max)
                .map(|(x, y)| Cross::new((*x, *y), 6, RED)),
        )?;
    }

    root_area.present().expect("Unable to write result to file");

    // print some information
    println!("Points count: {}", data.len());
    println!("Distinct size point count: {}", original_data.len());
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

fn find_y_max(
    lin_slope_1: f32,
    lin_intercept_1: f32,
    lin_slope_2: Option<f32>,
    lin_intercept_2: Option<f32>,
    x_max: f32,
    y_max: f32,
) -> f32 {
    let mut y_max_ret = y_max;
    let lin_1_y_max = if lin_slope_1 > 0f32 {
        lin_slope_1 * x_max + lin_intercept_1
    } else {
        lin_intercept_1
    };
    let lin_2_y_max = if lin_slope_2.is_some() {
        if lin_slope_2.unwrap() > 0f32 {
            lin_slope_2.unwrap() * x_max + lin_intercept_2.unwrap()
        } else {
            lin_intercept_2.unwrap()
        }
    } else {
        0f32
    };
    if lin_1_y_max > y_max {
        y_max_ret = lin_1_y_max;
    }
    if lin_slope_2.is_some() && lin_2_y_max > y_max {
        y_max_ret = lin_2_y_max;
    }
    y_max_ret
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
    let (lin_slope_1, lin_intercept_1): (f32, f32) = linear_regression_of(&v1).unwrap_or_default();

    // calculate linethrough 1st and last diff points
    let v2: Vec<(f32, f32)> = vec![
        *data_series1.first().unwrap_or(&(0f32, 0f32)),
        *data_series1.last().unwrap_or(&(0f32, 0f32)),
    ];
    let (lin_slope_2, lin_intercept_2): (f32, f32) = linear_regression_of(&v2).unwrap_or_default();

    // calculate axis max/min values
    let y_ofs = 0f32;
    let x_ofs = 0f32;
    let x_min = -x_ofs;
    let x_max = data_series1
        .iter()
        .map(|i| i.0 as i32)
        .max()
        .unwrap_or_default() as f32
        + x_ofs;
    let y_min = 0f32;
    let mut y_max = data_series1
        .iter()
        .map(|i| i.1 as i32)
        .max()
        .unwrap_or_default() as f32
        + y_ofs;
    let y_max_2 = data_series2
        .iter()
        .map(|i| i.1 as i32)
        .max()
        .unwrap_or_default() as f32
        + y_ofs;
    if y_max_2 > y_max {
        y_max = y_max_2;
    }

    y_max = find_y_max(
        lin_slope_1,
        lin_intercept_1,
        Some(lin_slope_2),
        Some(lin_intercept_2),
        x_max,
        y_max,
    ) * 1.1f32;

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
        .y_desc("DB read duration [µs]")
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
                .map(|x| (x, lin_slope_1 * x + lin_intercept_1)),
            &RED,
        ))?
        .label(format!(
            "Linear approx. of diff points: f(x)={:.3e}*x{}",
            lin_slope_1,
            number_to_str!(lin_intercept_1, 1)
        ))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    scatter_ctx
        .draw_series(LineSeries::new(
            lin_x_axis
                .values()
                .map(|x| (x, lin_slope_2 * x + lin_intercept_2)),
            &BLACK,
        ))?
        .label(format!(
            "Line by 1st and last RocksDB point: f(x)={:.3e}*x{}",
            lin_slope_2,
            number_to_str!(lin_intercept_2, 1)
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

    // print some information
    println!("Points list (size, time[µs]): {:?}", v1);
    println!(
        "Linear approx.:  f(size) = {} * size + {}\n",
        lin_slope_1, lin_intercept_1
    );
    println!(
        "Liny by 1st and last RocksDB point:  f(size) = {} * size + {}\n",
        lin_slope_2, lin_intercept_2
    );

    Ok(())
}

pub fn export_graph_two_series(
    caption: &str,
    data_series1: &Vec<(f32, f32)>,
    data_series2: &Vec<(f32, f32)>,
    output_png_file: &str,
    x_axis_label: &str,
    y_axis_label: &str,
    label1: &str,
    label2: &str,
    (lin_slope_1, lin_intercept_1): (f32, f32),
    (lin_slope_2, lin_intercept_2): (f32, f32),
) -> Result<(), Box<dyn std::error::Error>> {
    // calculate axis max/min values
    let (x_min, x_max, mut y_min, _y_max) =
        calculate_axis_ranges_for_two_series(&data_series1, &data_series2, None, None);
    if lin_intercept_1 < 0f32 && lin_intercept_1 < y_min {
        y_min = lin_intercept_1;
    }
    if lin_intercept_2 < 0f32 && lin_intercept_2 < y_min {
        y_min = lin_intercept_2;
    }
    let line_max_y_1 = lin_slope_1 * x_max + lin_intercept_1;
    let line_max_y_2 = lin_slope_2 * x_max + lin_intercept_2;
    let y_max: f32 = if line_max_y_1 > line_max_y_2 {
        line_max_y_1 * 1.2f32
    } else {
        line_max_y_2 * 1.2f32
    };

    let lin_x_axis = (x_min..(x_max + 1f32)).step(1f32);

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
        .x_desc(x_axis_label)
        .y_desc(y_axis_label)
        .axis_desc_style(("sans-serif", 16))
        .draw()?;
    // 1. draw read series1 points
    scatter_ctx
        .draw_series(
            data_series1
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
        )?
        .label(label1)
        .legend(|(x, y)| Circle::new((x + 10, y), 2, GREEN.filled()));
    // 2. draw read series2 points
    scatter_ctx
        .draw_series(
            data_series2
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 2, BLUE.filled())),
        )?
        .label(label2)
        .legend(|(x, y)| Circle::new((x + 10, y), 2, BLUE.filled()));
    // 3. draw linear approximetion line
    scatter_ctx
        .draw_series(LineSeries::new(
            lin_x_axis
                .values()
                .map(|x| (x, lin_slope_1 * x + lin_intercept_1)),
            &RED,
        ))?
        .label(format!(
            "Linear approx. of series 1 f(x)={:.3e}*x{}",
            lin_slope_1,
            number_to_str!(lin_intercept_1, 1)
        ))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    scatter_ctx
        .draw_series(LineSeries::new(
            lin_x_axis
                .values()
                .map(|x| (x, lin_slope_2 * x + lin_intercept_2)),
            &BLACK,
        ))?
        .label(format!(
            "Linear approx. of series 2 f(x)={:.3e}*x{}",
            lin_slope_2,
            number_to_str!(lin_intercept_2, 1)
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

pub fn export_graph_and_print_summary_read_not_found_results<
    S: SubstateDatabase + CommittableSubstateDatabase,
>(
    substate_store: &SubstateStoreWithMetrics<S>,
    output_png_file: &str,
    caption: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = &substate_store.read_not_found_metrics.borrow();

    let mut use_micro = true;
    let mut y_desc = "Duration [µs]";

    // calculate axis max/min values
    let x_min = 0;
    let x_max = data.len();
    let y_min = 0;
    let mut y_max = data
        .iter()
        .max()
        .unwrap_or(&Duration::default())
        .as_micros();
    if y_max == 0 {
        y_max = data.iter().max().unwrap_or(&Duration::default()).as_nanos();
        use_micro = false;
        y_desc = "Duration [ns]"
    }
    y_max += 10;

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
        .x_desc("Count")
        .y_desc(y_desc)
        .axis_desc_style(("sans-serif", 16))
        .draw()?;
    // 1. draw all read points
    scatter_ctx
        .draw_series(data.iter().enumerate().map(|(idx, y)| {
            let value = if use_micro {
                y.as_micros()
            } else {
                y.as_nanos()
            };
            Circle::new((idx, value), 2, GREEN.filled())
        }))?
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

    // print some information
    let (avg, unit) = if use_micro {
        (
            data.iter().sum::<Duration>().as_micros() as usize / data.len(),
            'µ',
        )
    } else {
        (
            data.iter().sum::<Duration>().as_nanos() as usize / data.len(),
            'n',
        )
    };
    println!(
        "Read not found times [{}s]: min={} max={} avg={}",
        unit, y_min, y_max, avg
    );
    println!("Points count: {}", data.len());
    println!("Output graph file: {}\n", output_png_file);

    Ok(())
}
