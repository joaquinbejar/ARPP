/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 10/9/24
 ******************************************************************************/

use rust_decimal::Decimal;
use tracing::info;
use arpp::analysis::metrics::{PoolMetrics, SimulationAnalysis};
use arpp::analysis::visualization::{create_price_chart, create_metrics_chart, create_simulation_analysis_chart};
use arpp::utils::logger::setup_logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let prices = vec![
        Decimal::new(100, 2),  // 1.00
        Decimal::new(102, 2),  // 1.02
        Decimal::new(98, 2),   // 0.98
        Decimal::new(105, 2),  // 1.05
        Decimal::new(103, 2),  // 1.03
    ];
    create_price_chart(&prices, "draws/price_chart.png")?;
    info!("Price chart created: draws/price_chart.png");

    // Ejemplo para create_metrics_chart
    let metrics = vec![
        PoolMetrics {
            price_volatility: Decimal::new(5, 3),    // 0.005
            liquidity_depth: Decimal::new(1000, 0),  // 1000
            trading_volume: Decimal::new(5000, 0),   // 5000
            impermanent_loss: Decimal::new(-2, 2),   // -0.02
        },
        PoolMetrics {
            price_volatility: Decimal::new(7, 3),    // 0.007
            liquidity_depth: Decimal::new(1100, 0),  // 1100
            trading_volume: Decimal::new(5500, 0),   // 5500
            impermanent_loss: Decimal::new(-25, 3),  // -0.025
        },
    ];

    create_metrics_chart(&metrics, "draws/metrics_chart.png")?;
    info!("Metrics chart created: draws/metrics_chart.png");

    // Ejemplo para create_simulation_analysis_chart
    let analysis = SimulationAnalysis {
        price_stability: Decimal::new(95, 2),    // 0.95
        average_price_impact: Decimal::new(2, 2), // 0.02
        liquidity_efficiency: Decimal::new(98, 2), // 0.98
    };
    create_simulation_analysis_chart(&analysis, "draws/simulation_analysis_chart.png")?;
    info!("Simulation analysis chart created: draws/simulation_analysis_chart.png");

    Ok(())
}