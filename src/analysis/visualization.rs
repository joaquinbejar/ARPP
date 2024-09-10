/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use plotters::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::analysis::metrics::{PoolMetrics, SimulationAnalysis};

pub fn create_price_chart(prices: &[Decimal], file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_price = prices.iter().min().unwrap();
    let max_price = prices.iter().max().unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Price Evolution", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..prices.len() as f32, min_price.to_f32().unwrap()..max_price.to_f32().unwrap())?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        prices.iter().enumerate().map(|(i, &price)| (i as f32, price.to_f32().unwrap())),
        &RED,
    ))?;

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