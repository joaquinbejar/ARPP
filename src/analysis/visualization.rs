/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use crate::analysis::metrics::{PoolMetrics, SimulationAnalysis};
use plotters::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::cmp;
use tracing::info;

/// Creates a price chart and saves it to an image file.
///
/// # Arguments
///
/// * `prices` - A slice of `Decimal` values representing the price points.
/// * `p_refs` - A slice of `Decimal` values representing the reference price points.
/// * `file_name` - The name of the file where the chart will be saved.
/// * `alpha` - A `Decimal` value used for additional details in the chart title.
/// * `beta` - A `Decimal` value used for additional details in the chart title.
///
/// # Returns
///
/// This function returns a `Result` which is `Ok` if the operation succeeded,
/// or an `Err` containing a boxed `dyn std::error::Error` if the operation failed.
///
/// # Errors
///
/// This function can return an error if:
/// - There is a problem creating or saving the image file.
/// - The min and max operations on the slices fail.
/// - Any of the chart drawing operations fail.
///
pub fn create_price_chart(
    prices: &[Decimal],
    p_refs: &[Decimal],
    file_name: &str,
    alpha: Decimal,
    beta: Decimal,
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

    let title = format!("Price Evolution (alpha: {}, beta: {})", alpha, beta);
    // Build the chart with the combined min/max for y-axis
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            0f32..prices.len() as f32,
            overall_min_price.to_f32().unwrap()..overall_max_price.to_f32().unwrap(),
        )?;

    chart.configure_mesh().draw()?;

    // Draw the price line in RED
    chart
        .draw_series(LineSeries::new(
            prices
                .iter()
                .enumerate()
                .map(|(i, &price)| (i as f32, price.to_f32().unwrap())),
            &RED,
        ))?
        .label("Prices")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    // Draw the reference price line (p_refs) in BLUE
    chart
        .draw_series(LineSeries::new(
            p_refs
                .iter()
                .enumerate()
                .map(|(i, &p_ref)| (i as f32, p_ref.to_f32().unwrap())),
            &BLUE,
        ))?
        .label("Reference Prices")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    // Configure the legend
    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    // Save the chart
    root.present()?;
    Ok(())
}

/// Creates a metrics chart and saves it to a file.
///
/// This function takes a slice of `PoolMetrics` and generates a chart that visualizes the metrics over time.
/// It saves the generated chart to the specified file name.
///
/// # Arguments
///
/// * `metrics` - A slice of `PoolMetrics` containing the data to be visualized in the chart.
/// * `file_name` - A string slice that holds the name of the file where the chart will be saved.
///
/// # Returns
///
/// This function returns a `Result` which is:
/// * `Ok(())` if the chart was successfully created and saved.
/// * `Err` if there was an error during the creation or saving of the chart.
///
/// # Errors
///
/// This function can return errors related to drawing operations or file-related issues.
///
/// # Dependencies
///
/// This function requires the `plotters` crate to generate charts.
///
/// ```toml
/// [dependencies]
/// plotters = "0.3.7"
/// ```
///
pub fn create_metrics_chart(
    metrics: &[PoolMetrics],
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Pool Metrics Over Time", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..metrics.len() as f32, 0f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            metrics
                .iter()
                .enumerate()
                .map(|(i, m)| (i as f32, m.price_volatility.to_f32().unwrap())),
            &RED,
        ))?
        .label("Price Volatility")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    chart
        .draw_series(LineSeries::new(
            metrics
                .iter()
                .enumerate()
                .map(|(i, m)| (i as f32, m.liquidity_depth.to_f32().unwrap())),
            &BLUE,
        ))?
        .label("Liquidity Depth")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart.configure_series_labels().border_style(BLACK).draw()?;

    root.present()?;
    Ok(())
}

/// Creates a simulation analysis chart and saves it to a file.
///
/// The chart visualizes the following metrics from the simulation analysis:
/// - Price Stability
/// - Average Price Impact
/// - Liquidity Efficiency
///
/// # Arguments
/// * `analysis` - A reference to the `SimulationAnalysis` containing the data to be plotted.
/// * `file_name` - The name of the file to save the chart to.
/// * `alpha` - A Decimal value representing the alpha parameter used in the simulation.
/// * `beta` - A Decimal value representing the beta parameter used in the simulation.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Returns `Ok` if the chart is successfully created and saved; otherwise, returns an error.
///
/// # Errors
/// This function will return an error if any of the following operations fail:
/// * Creating the drawing area for the chart.
/// * Filling the drawing area with the background color.
/// * Building the chart with the provided data.
/// * Drawing the chart's mesh.
/// * Writing text onto the chart.
/// * Saving the chart to a file.
///
pub fn create_simulation_analysis_chart(
    analysis: &SimulationAnalysis,
    file_name: &str,
    alpha: Decimal,
    beta: Decimal,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let title = format!("Simulation Analysis (alpha: {}, beta: {})", alpha, beta);
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..3, 0f32..1f32)?;

    chart.configure_mesh().draw()?;

    let data = [
        (
            "Price Stability",
            analysis.price_stability.to_f32().unwrap_or(0.0),
        ),
        (
            "Avg Price Impact",
            analysis.average_price_impact.to_f32().unwrap_or(0.0),
        ),
        (
            "Liquidity Efficiency",
            analysis.liquidity_efficiency.to_f32().unwrap_or(0.0),
        ),
    ];

    chart.draw_series(data.iter().enumerate().map(|(i, (_label, value))| {
        let mut bar = Rectangle::new([(i as i32, 0.0), ((i as i32) + 1, *value)], RED.filled());
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

/// Visualizes a random walk of prices and saves the plot to a specified file.
///
/// # Arguments
///
/// * `prices` - A vector of `Decimal` values representing the sequence of prices.
/// * `output_file` - A string slice that holds the name of the file to output the plot to.
///
/// # Returns
///
/// This function returns a `Result` indicating the success or failure of the operation.
/// It returns `Ok(())` if the plot is successfully saved, otherwise it returns an error.
///
/// # Errors
///
/// This function will return an error if any of the following occurs:
/// - Converting `Decimal` to `f64` fails.
/// - Errors in setting up or drawing the plot.
/// - Fails to save the chart as an image.
///
pub fn visualize_random_walk(
    prices: Vec<Decimal>,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert the prices to f64 to work with plotters.
    let price_values: Vec<f64> = prices.iter().map(|p| p.to_f64().unwrap()).collect();

    // Calculate the minimum and maximum price in the sequence.
    let min_price = price_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_price = price_values
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

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
    chart
        .configure_mesh()
        .x_desc("Steps")
        .y_desc("Price")
        .draw()?;

    // Convert prices to f64 for plotting.
    let price_points: Vec<(usize, f64)> = prices
        .iter()
        .enumerate()
        .map(|(i, p)| (i, p.to_f64().unwrap()))
        .collect();

    // Draw the line series for the prices.
    chart
        .draw_series(LineSeries::new(price_points, &BLUE))?
        .label("Price")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    // Add the legend to the chart.
    chart.configure_series_labels().border_style(BLACK).draw()?;

    // Save the chart as an image.
    root.present()?;
    info!("Plot saved to {}", output_file);

    Ok(())
}

/// Visualizes a sequence of random walks by generating a plot and saving it as an image file.
///
/// # Arguments
///
/// * `sequences` - A vector of vectors, where each inner vector contains `Decimal` values
///   representing a random walk sequence.
/// * `output_file` - A string slice that holds the path to the output image file.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - Returns an empty `Ok` if successful, or an error
///   if any occurs during the plotting process.
///
pub fn visualize_random_walks(
    sequences: Vec<Vec<Decimal>>,
    output_file: &str,
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
        .caption(
            "Multiple Random Walk Sequences",
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..sequences[0].len(), min_price..max_price)?;

    // Label the chart axes.
    chart
        .configure_mesh()
        .x_desc("Steps")
        .y_desc("Price")
        .draw()?;

    // Define a set of colors for the different sequences
    let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN, &YELLOW, &BLACK];

    // Plot each sequence with a different color
    for (i, seq) in sequences_f64.iter().enumerate() {
        let color = colors[i % colors.len()]; // Rotate colors if there are more sequences than colors

        let price_points: Vec<(usize, f64)> =
            seq.iter().enumerate().map(|(x, &y)| (x, y)).collect();

        chart.draw_series(LineSeries::new(price_points, color))?;
    }

    // Save the chart as an image.
    root.present()?;
    info!("Plot saved to {}", output_file);

    Ok(())
}

#[cfg(test)]
mod tests_graphs {
    use super::*;

    use tempfile::tempdir;

    // A helper function to check the existence of the file
    fn file_exists(file_path: &str) -> bool {
        std::path::Path::new(file_path).exists()
    }

    #[test]
    fn test_create_price_chart() {
        // Setup temporary directory
        let dir = tempdir().unwrap();
        let file_path = dir
            .path()
            .join("price_chart.png")
            .to_str()
            .unwrap()
            .to_string();

        // Arrange
        let prices = vec![
            Decimal::new(100, 2),
            Decimal::new(110, 2),
            Decimal::new(115, 2),
        ];
        let p_refs = vec![
            Decimal::new(90, 2),
            Decimal::new(95, 2),
            Decimal::new(100, 2),
        ];
        let alpha = Decimal::new(1, 2);
        let beta = Decimal::new(2, 2);

        // Act
        let result = create_price_chart(&prices, &p_refs, &file_path, alpha, beta);

        // Assert
        assert!(result.is_ok(), "Expected Ok but got Err");
        assert!(file_exists(&file_path), "Expected file to exist");
    }

    #[test]
    fn test_create_metrics_chart() {
        // Setup temporary directory
        let dir = tempdir().unwrap();
        let file_path = dir
            .path()
            .join("metrics_chart.png")
            .to_str()
            .unwrap()
            .to_string();

        // Arrange
        let metrics = vec![
            PoolMetrics {
                steps: vec![],
                price_volatility: Decimal::new(1, 2),
                liquidity_depth: Decimal::new(2, 2),
                trading_volume: Default::default(),
                impermanent_loss: Default::default(),
            },
            PoolMetrics {
                steps: vec![],
                price_volatility: Decimal::new(3, 2),
                liquidity_depth: Decimal::new(4, 2),
                trading_volume: Default::default(),
                impermanent_loss: Default::default(),
            },
        ];

        // Act
        let result = create_metrics_chart(&metrics, &file_path);

        // Assert
        assert!(result.is_ok(), "Expected Ok but got Err");
        assert!(file_exists(&file_path), "Expected file to exist");
    }

    #[test]
    fn test_create_simulation_analysis_chart() {
        // Setup temporary directory
        let dir = tempdir().unwrap();
        let file_path = dir
            .path()
            .join("simulation_analysis_chart.png")
            .to_str()
            .unwrap()
            .to_string();

        // Arrange
        let analysis = SimulationAnalysis {
            price_stability: Decimal::new(5, 2),
            average_price_impact: Decimal::new(6, 2),
            liquidity_efficiency: Decimal::new(7, 2),
        };
        let alpha = Decimal::new(1, 2);
        let beta = Decimal::new(2, 2);

        // Act
        let result = create_simulation_analysis_chart(&analysis, &file_path, alpha, beta);

        // Assert
        assert!(result.is_ok(), "Expected Ok but got Err");
        assert!(file_exists(&file_path), "Expected file to exist");
    }
}
