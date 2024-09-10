/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use plotters::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use tracing::info;
use crate::analysis::metrics::{PoolMetrics, SimulationAnalysis};
use std::cmp;

/// Creates a price chart with both prices and reference prices (`p_refs`).
///
/// # Arguments:
/// - `prices`: A slice of `Decimal` representing the prices over time.
/// - `p_refs`: A slice of `Decimal` representing the reference prices over time.
/// - `file_name`: The file path where the chart will be saved.
///
/// # Returns:
/// A result indicating success or failure.
pub fn create_price_chart(
    prices: &[Decimal],
    p_refs: &[Decimal],
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the drawing area
    let root = BitMapBackend::new(file_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the minimum and maximum values between both `prices` and `p_refs`
    let min_price_prices = prices.iter().chain(p_refs.iter()).min().unwrap();
    let max_price_prices = prices.iter().chain(p_refs.iter()).max().unwrap();
    let min_price_p_ref = p_refs.iter().chain(p_refs.iter()).min().unwrap();
    let max_price_p_ref = p_refs.iter().chain(p_refs.iter()).max().unwrap();

    let overall_min_price = cmp::min(min_price_prices, min_price_p_ref);
    let overall_max_price = cmp::max(max_price_prices, max_price_p_ref);

    // Build the chart with the combined min/max for y-axis
    let mut chart = ChartBuilder::on(&root)
        .caption("Price Evolution", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            0f32..prices.len() as f32,
            overall_min_price.to_f32().unwrap()..overall_max_price.to_f32().unwrap()
        )?;

    chart.configure_mesh().draw()?;

    // Draw the price line in RED
    chart.draw_series(LineSeries::new(
        prices.iter().enumerate().map(|(i, &price)| (i as f32, price.to_f32().unwrap())),
        &RED,
    ))?
        .label("Prices")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    // Draw the reference price line (p_refs) in BLUE
    chart.draw_series(LineSeries::new(
        p_refs.iter().enumerate().map(|(i, &p_ref)| (i as f32, p_ref.to_f32().unwrap())),
        &BLUE,
    ))?
        .label("Reference Prices")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Configure the legend
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    // Save the chart
    root.present()?;
    Ok(())
}
pub fn create_metrics_chart(metrics: &[PoolMetrics], file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Pool Metrics Over Time", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..metrics.len() as f32, 0f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        metrics.iter().enumerate().map(|(i, m)| (i as f32, m.price_volatility.to_f32().unwrap())),
        &RED,
    ))?.label("Price Volatility").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.draw_series(LineSeries::new(
        metrics.iter().enumerate().map(|(i, m)| (i as f32, m.liquidity_depth.to_f32().unwrap())),
        &BLUE,
    ))?.label("Liquidity Depth").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart.configure_series_labels().border_style(&BLACK).draw()?;

    root.present()?;
    Ok(())
}

pub fn create_simulation_analysis_chart(analysis: &SimulationAnalysis, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Simulation Analysis", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..3, 0f32..1f32)?;

    chart.configure_mesh().draw()?;

    let data = [
        ("Price Stability", analysis.price_stability.to_f32().unwrap_or(0.0)),
        ("Avg Price Impact", analysis.average_price_impact.to_f32().unwrap_or(0.0)),
        ("Liquidity Efficiency", analysis.liquidity_efficiency.to_f32().unwrap_or(0.0)),
    ];

    chart.draw_series(data.iter().enumerate().map(|(i, (_label, value))| {
        let mut bar = Rectangle::new([(i as i32, 0.0), ((i as i32) + 1, value.clone())], RED.filled());
        bar.set_margin(0, 0, 5, 5);
        bar
    }))?;

    for (i, (label, _)) in data.iter().enumerate() {
        root.draw_text(
            label,
            &TextStyle::from(("sans-serif", 20).into_font()).color(&BLACK),
            ((40 + i * 260) as i32, 550),
        )?;
    }

    root.present()?;
    Ok(())
}

/// Visualizes a random walk sequence by plotting the prices over time.
///
/// # Arguments:
///
/// * `prices` - A vector of `Decimal` prices representing the random walk sequence.
/// * `output_file` - The file path where the plot image will be saved.
///
/// # Returns:
///
/// Saves the plot as an image file (PNG).
pub fn visualize_random_walk(prices: Vec<Decimal>, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Convert the prices to f64 to work with plotters.
    let price_values: Vec<f64> = prices.iter().map(|p| p.to_f64().unwrap()).collect();

    // Calculate the minimum and maximum price in the sequence.
    let min_price = price_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_price = price_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Set up the drawing area for the plot (800x600 image).
    let root = BitMapBackend::new(output_file, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Define the chart area and labels, adjusting Y axis with min and max values.
    let mut chart = ChartBuilder::on(&root)
        .caption("Random Walk Price Sequence", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..prices.len(), min_price..max_price)?;

    // Label the chart axes.
    chart.configure_mesh().x_desc("Steps").y_desc("Price").draw()?;

    // Convert prices to f64 for plotting.
    let price_points: Vec<(usize, f64)> = prices
        .iter()
        .enumerate()
        .map(|(i, p)| (i, p.to_f64().unwrap()))
        .collect();

    // Draw the line series for the prices.
    chart.draw_series(LineSeries::new(price_points, &BLUE))?
        .label("Price")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Add the legend to the chart.
    chart.configure_series_labels().border_style(&BLACK).draw()?;

    // Save the chart as an image.
    root.present()?;
    info!("Plot saved to {}", output_file);

    Ok(())
}


/// Visualizes multiple random walk sequences by plotting each sequence with a different color.
///
/// # Arguments:
///
/// * `sequences` - A vector of sequences (each sequence is a vector of `Decimal` prices).
/// * `output_file` - The file path where the plot image will be saved.
///
/// # Returns:
///
/// Saves the plot as an image file (PNG).
pub fn visualize_random_walks(
    sequences: Vec<Vec<Decimal>>,
    output_file: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up the drawing area for the plot (800x600 image).
    let root = BitMapBackend::new(output_file, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Convert all prices to f64 for plotting
    let sequences_f64: Vec<Vec<f64>> = sequences
        .iter()
        .map(|seq| seq.iter().map(|p| p.to_f64().unwrap()).collect())
        .collect();

    // Calculate the minimum and maximum price across all sequences
    let min_price = sequences_f64
        .iter()
        .flat_map(|seq| seq.iter())
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let max_price = sequences_f64
        .iter()
        .flat_map(|seq| seq.iter())
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

    // Define the chart area and labels, adjusting Y axis with min and max values.
    let mut chart = ChartBuilder::on(&root)
        .caption("Multiple Random Walk Sequences", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..sequences[0].len(), min_price..max_price)?;

    // Label the chart axes.
    chart.configure_mesh().x_desc("Steps").y_desc("Price").draw()?;

    // Define a set of colors for the different sequences
    let colors = vec![&RED, &BLUE, &GREEN, &MAGENTA, &CYAN, &YELLOW, &BLACK];

    // Plot each sequence with a different color
    for (i, seq) in sequences_f64.iter().enumerate() {
        let color = colors[i % colors.len()]; // Rotate colors if there are more sequences than colors

        let price_points: Vec<(usize, f64)> = seq.iter().enumerate().map(|(x, &y)| (x, y)).collect();

        chart
            .draw_series(LineSeries::new(price_points, color))?;
    }


    // Save the chart as an image.
    root.present()?;
    info!("Plot saved to {}", output_file);

    Ok(())
}