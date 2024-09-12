/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/
use crate::analysis::metrics::PoolMetrics;
use crate::simulation::monte_carlo::MonteCarloSimulation;
use rust_decimal::Decimal;
use std::error::Error;
use std::time::Duration;

/// Represents the result of a simulation, including various metrics such as
/// average price change, average liquidity change, maximum price, minimum price,
/// and a set of additional pool metrics.
///
/// # Fields
///
/// * `average_price_change` - The average change in price during the simulation.
/// * `average_liquidity_change` - The average change in liquidity during the simulation.
/// * `max_price` - The maximum price recorded during the simulation.
/// * `min_price` - The minimum price recorded during the simulation.
/// * `metrics` - A collection of additional metrics related to the pool performance during the simulation.
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
    /// Constructs a new `SimulationResult`.
    ///
    /// # Arguments
    ///
    /// * `average_price_change` - Decimal value representing the average price change.
    /// * `average_liquidity_change` - Decimal value representing the average liquidity change.
    /// * `max_price` - Decimal value representing the maximum price reached in the simulation.
    /// * `min_price` - Decimal value representing the minimum price reached in the simulation.
    /// * `metrics` - An instance of `PoolMetrics` containing additional metric information.
    ///
    /// # Returns
    ///
    /// * A new instance of `SimulationResult`.
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

#[cfg(test)]
mod tests_simulation_result {
    use super::*;
    use crate::analysis::metrics::PoolMetrics;
    use tokio;

    #[tokio::test]
    async fn test_default_simulation_result() {
        let default_result = SimulationResult::default();

        assert_eq!(default_result.average_price_change, Decimal::ZERO);
        assert_eq!(default_result.average_liquidity_change, Decimal::ZERO);
        assert_eq!(default_result.max_price, Decimal::ZERO);
        assert_eq!(default_result.min_price, Decimal::ZERO);
        assert_eq!(default_result.metrics, PoolMetrics::default());
    }

    #[tokio::test]
    async fn test_custom_simulation_result() {
        let custom_metrics = PoolMetrics::default();
        let custom_result = SimulationResult::new(
            Decimal::new(50, 1),  // 5.0
            Decimal::new(25, 1),  // 2.5
            Decimal::new(100, 1), // 10.0
            Decimal::new(10, 1),  // 1.0
            custom_metrics.clone(),
        );

        assert_eq!(custom_result.average_price_change, Decimal::new(50, 1));
        assert_eq!(custom_result.average_liquidity_change, Decimal::new(25, 1));
        assert_eq!(custom_result.max_price, Decimal::new(100, 1));
        assert_eq!(custom_result.min_price, Decimal::new(10, 1));
        assert_eq!(custom_result.metrics, custom_metrics);
    }
}
