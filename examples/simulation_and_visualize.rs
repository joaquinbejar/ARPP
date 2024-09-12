/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/
use arpp::analysis::metrics::analyze_simulation_results;
use arpp::analysis::visualization::{
    create_metrics_chart, create_price_chart, create_simulation_analysis_chart,
};
use arpp::arpp::liquidity_pool::LiquidityPool;
use arpp::simulation::monte_carlo::MonteCarloSimulation;
use arpp::simulation::strategies::MeanReversionStrategy;
use arpp::utils::logger::setup_logger;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let alpha = dec!(0.8);
    let beta = dec!(5);

    let initial_pool = LiquidityPool::new(
        dec!(100000), // token_a
        dec!(100000), // token_b
        dec!(100),    // p_ref
        alpha,        // alpha
        beta,         // beta
    );

    let strategy = Box::new(MeanReversionStrategy::new(dec!(0.0004), dec!(0.22)));
    let iterations = 1000;
    let steps_per_iteration = 1;

    let mut simulation = MonteCarloSimulation::new(
        initial_pool.clone(),
        iterations,
        steps_per_iteration,
        strategy,
        Decimal::new(1, 1),
        Decimal::new(1, 2),
    );

    let result = simulation.run().await?;

    info!("Simulation completed");
    info!("\tAverage price change: {:.4}", result.average_price_change);
    info!(
        "\tAverage liquidity change: {:.4}",
        result.average_liquidity_change
    );
    info!("\tMax price: {:.4}", result.max_price);
    info!("\tMin price: {:.4}", result.min_price);

    let price_history = result.metrics.get_prices();
    let p_ref_history = result.metrics.get_p_ref();
    create_price_chart(
        &price_history,
        &p_ref_history,
        "draws/price_chart.png",
        alpha,
        beta,
    )?;
    info!("Price chart created: draws/price_chart.png");

    let metrics_history = simulation.get_metrics_history();
    create_metrics_chart(&metrics_history, "draws/metrics_chart.png")?;
    info!("Metrics chart created: draws/metrics_chart.png");

    let pool_metrics = result.clone().metrics;
    let analysis = analyze_simulation_results(&result);
    create_simulation_analysis_chart(
        &analysis,
        "draws/simulation_analysis_chart.png",
        alpha,
        beta,
    )?;
    info!("Simulation analysis chart created: draws/simulation_analysis_chart.png");

    info!("Final Pool Metrics:");
    info!("\tPrice Volatility: {:.4}", pool_metrics.price_volatility);
    info!("\tLiquidity Depth: {:.4}", pool_metrics.liquidity_depth);
    info!("\tTrading Volume: {:.4}", pool_metrics.trading_volume);
    info!("\tImpermanent Loss: {:.4}", pool_metrics.impermanent_loss);

    info!("Simulation Analysis:");
    info!("\tPrice Stability: {:.4}", analysis.price_stability);
    info!("\tAverage Price Impact: {:.4}", analysis.average_price_impact);
    info!("\tLiquidity Efficiency: {:.4}", analysis.liquidity_efficiency);

    Ok(())
}
