/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 10/9/24
 ******************************************************************************/
use std::error::Error;
use std::time::Duration;
use rust_decimal::Decimal;
use crate::analysis::metrics::PoolMetrics;
use crate::simulation::monte_carlo::MonteCarloSimulation;

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub average_price_change: Decimal,
    pub average_liquidity_change: Decimal,
    pub max_price: Decimal,
    pub min_price: Decimal,
    pub metrics: PoolMetrics,
}

impl Default for SimulationResult {
    fn default() -> Self {
        SimulationResult {
            average_price_change: Decimal::ZERO,
            average_liquidity_change: Decimal::ZERO,
            max_price: Decimal::ZERO,
            min_price: Decimal::ZERO,
            metrics: PoolMetrics::default(),
        }
    }
}

impl SimulationResult {
    pub fn new(
        average_price_change: Decimal,
        average_liquidity_change: Decimal,
        max_price: Decimal,
        min_price: Decimal,
        metrics: PoolMetrics,
    ) -> Self {
        Self {
            average_price_change,
            average_liquidity_change,
            max_price,
            min_price,
            metrics,
        }
    }
}


/// Runs a timed Monte Carlo simulation asynchronously.
///
/// This function starts a timer before running the provided Monte Carlo simulation
/// asynchronously. Once the simulation completes, it calculates the elapsed time and
/// returns the simulation result along with the duration taken to complete the simulation.
///
/// # Arguments
///
/// * `simulation` - A reference to an instance of `MonteCarloSimulation` which needs to be run.
///
/// # Returns
///
/// This function returns a `Result` which, on success, contains a tuple with:
///
/// * `SimulationResult` - The result of the Monte Carlo simulation.
/// * `Duration` - The duration it took to run the simulation.
///
/// On failure, it returns a boxed `dyn Error` indicating what went wrong during the simulation.
///
/// # Errors
///
/// This function will return an error if the simulation fails to run properly.
///
pub async fn run_timed_simulation(
    simulation: &mut MonteCarloSimulation,
) -> Result<(SimulationResult, Duration), Box<dyn Error>> {
    let start = std::time::Instant::now();
    let result = simulation.run().await?;
    let duration = start.elapsed();
    Ok((result, duration))
}