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

    let mut initial_pool = LiquidityPool::new(
        dec!(100000),  // token_a
        dec!(100000),  // token_b
        dec!(100),       // p_ref
        dec!(0.5),     // alpha
        dec!(1),       // beta
    );

    // let strategy = Box::new(RandomStrategy::new(0.5, dec!(10)));
    let strategy = Box::new(MeanReversionStrategy::new(dec!(0.1), dec!(100)));
    let iterations = 100;
    let steps_per_iteration = 10;

    // Crear y ejecutar la simulación
    let mut simulation = MonteCarloSimulation::new(
        initial_pool.clone(),
        iterations,
        steps_per_iteration,
        strategy,
        Decimal::new(1, 1),   // Parámetros de simulación adicionales si son necesarios
        Decimal::new(1, 2)
    );

    let result = simulation.run().await?;

    info!("Simulation completed");
    info!("Average price change: {}", result.average_price_change);
    info!("Average liquidity change: {}", result.average_liquidity_change);
    info!("Max price: {}", result.max_price);
    info!("Min price: {}", result.min_price);

    // Generar gráficos
    // 1. Gráfico de precios
    let price_history = result.metrics.get_prices();
    let p_ref_history = result.metrics.get_p_ref();
    create_price_chart(&price_history, &p_ref_history, "draws/price_chart.png")?;
    info!("Price chart created: draws/price_chart.png");

    // 2. Gráfico de métricas
    let metrics_history = simulation.get_metrics_history();
    create_metrics_chart(&metrics_history, "draws/metrics_chart.png")?;
    info!("Metrics chart created: draws/metrics_chart.png");

    // 3. Análisis de la simulación
    let final_pool = simulation.get_final_pool();
    let pool_metrics = result.clone().metrics; // Usar las métricas del resultado de la simulación
    let analysis = analyze_simulation_results(&result);
    create_simulation_analysis_chart(&analysis, "draws/simulation_analysis_chart.png")?;
    info!("Simulation analysis chart created: draws/simulation_analysis_chart.png");

    // Imprimir métricas finales
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