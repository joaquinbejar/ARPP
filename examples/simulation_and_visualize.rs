/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 10/9/24
 ******************************************************************************/
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;
use arpp::utils::logger::setup_logger;
use arpp::arpp::liquidity_pool::LiquidityPool;
use arpp::simulation::monte_carlo::MonteCarloSimulation;
use arpp::simulation::strategies::{MeanReversionStrategy, RandomStrategy};
use arpp::analysis::metrics::analyze_simulation_results;
use arpp::analysis::visualization::{create_price_chart, create_metrics_chart, create_simulation_analysis_chart};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let alpha = dec!(0.2);
    let beta = dec!(5);

    let mut initial_pool = LiquidityPool::new(
        dec!(100000),  // token_a
        dec!(100000),  // token_b
        dec!(100),       // p_ref
        alpha,     // alpha
        beta,       // beta
    );

    // let strategy = Box::new(RandomStrategy::new(0.5, dec!(10)));
    let strategy = Box::new(MeanReversionStrategy::new(dec!(0.1), dec!(100)));
    let iterations = 1000;
    let steps_per_iteration = 10;


    let mut simulation = MonteCarloSimulation::new(
        initial_pool.clone(),
        iterations,
        steps_per_iteration,
        strategy,
        Decimal::new(1, 1),
        Decimal::new(1, 2)
    );

    let result = simulation.run().await?;

    info!("Simulation completed");
    info!("Average price change: {}", result.average_price_change);
    info!("Average liquidity change: {}", result.average_liquidity_change);
    info!("Max price: {}", result.max_price);
    info!("Min price: {}", result.min_price);


    let price_history = result.metrics.get_prices();
    let p_ref_history = result.metrics.get_p_ref();
    create_price_chart(&price_history, &p_ref_history, "draws/price_chart.png",alpha, beta)?;
    info!("Price chart created: draws/price_chart.png");

    let metrics_history = simulation.get_metrics_history();
    create_metrics_chart(&metrics_history, "draws/metrics_chart.png")?;
    info!("Metrics chart created: draws/metrics_chart.png");

    let final_pool = simulation.get_final_pool();
    let pool_metrics = result.clone().metrics;
    let analysis = analyze_simulation_results(&result);
    create_simulation_analysis_chart(&analysis, "draws/simulation_analysis_chart.png")?;
    info!("Simulation analysis chart created: draws/simulation_analysis_chart.png");

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