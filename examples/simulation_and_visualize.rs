/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 10/9/24
 ******************************************************************************/
use rust_decimal_macros::dec;
use tracing::info;
use arpp::utils::logger::setup_logger;
use arpp::arpp::liquidity_pool::LiquidityPool;
use arpp::simulation::monte_carlo::MonteCarloSimulation;
use arpp::simulation::strategies::RandomStrategy;
use arpp::analysis::metrics::{calculate_pool_metrics, analyze_simulation_results};
use arpp::analysis::visualization::{create_price_chart, create_metrics_chart, create_simulation_analysis_chart};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let initial_pool = LiquidityPool::new(
        dec!(100000),  // token_a
        dec!(100000),  // token_b
        dec!(1),     // p_ref
        dec!(0.5),   // alpha
        dec!(1),     // beta
    );

    let strategy = Box::new(RandomStrategy::new(0.5, dec!(10))); // 50% de probabilidad de swap, máximo 10 tokens
    let iterations = 10000;
    let steps_per_iteration = 1000;

    // Create and run the simulation
    let mut simulation = MonteCarloSimulation::new(initial_pool.clone(), iterations, steps_per_iteration, strategy);
    let result = simulation.run().await?;

    info!("Simulation completed");
    info!("Average price change: {}", result.average_price_change);
    info!("Average liquidity change: {}", result.average_liquidity_change);
    info!("Max price: {}", result.max_price);
    info!("Min price: {}", result.min_price);

    // Generate graphs
    // 1. Price graph
    let price_history = simulation.get_price_history();
    create_price_chart(&price_history, "draws/price_chart.png")?;
    info!("Price chart created: draws/price_chart.png");

    // 2. Metrics Chart
    let metrics_history = simulation.get_metrics_history();
    create_metrics_chart(&metrics_history, "draws/metrics_chart.png")?;
    info!("Metrics chart created: draws/metrics_chart.png");

    // 3. Simulation Analysis Graph
    let final_pool = simulation.get_final_pool();
    let pool_metrics = calculate_pool_metrics(&final_pool, &initial_pool);
    let analysis = analyze_simulation_results(&result);
    create_simulation_analysis_chart(&analysis, "draws/simulation_analysis_chart.png")?;
    info!("Simulation analysis chart created: draws/simulation_analysis_chart.png");

    // Print final metrics
    info!("\nFinal Pool Metrics:");
    info!("Price Volatility: {}", pool_metrics.price_volatility);
    info!("Liquidity Depth: {}", pool_metrics.liquidity_depth);
    info!("Trading Volume: {}", pool_metrics.trading_volume);
    info!("Impermanent Loss: {}", pool_metrics.impermanent_loss);

    info!("\nSimulation Analysis:");
    info!("Price Stability: {}", analysis.price_stability);
    info!("Average Price Impact: {}", analysis.average_price_impact);
    info!("Liquidity Efficiency: {}", analysis.liquidity_efficiency);

    Ok(())
}