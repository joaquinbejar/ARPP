/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use crate::arpp::liquidity_pool::LiquidityPool;
use crate::simulation::strategies::TradingStrategy;
use futures::future;
use futures::stream::{self, StreamExt};
use rust_decimal::Decimal;
use std::error::Error;
use std::time::Duration;
use tracing::error;

/// `MonteCarloSimulation` struct represents the settings and components required to perform a
/// Monte Carlo simulation for a liquidity pool trading strategy.
///
/// This struct provides a way to configure and run multiple iterations of a trading strategy over a specified
/// number of steps, starting with an initial liquidity pool.
///
/// ## Fields
///
/// - `initial_pool`: The initial state of the liquidity pool at the beginning of the simulation.
/// - `iterations`: The number of times the Monte Carlo simulation will be run.
/// - `steps_per_iteration`: The number of steps to be taken in each iteration of the simulation.
/// - `strategy`: The trading strategy that will be applied during the simulation.
///
/// ## Usage
///
/// `MonteCarloSimulation` allows you to set up a simulation with a specific trading strategy
/// and see how it performs over multiple iterations and steps, which can help in understanding
/// the robustness and performance of the strategy under various conditions.
///
pub struct MonteCarloSimulation {
    initial_pool: LiquidityPool,
    iterations: usize,
    steps_per_iteration: usize,
    strategy: Box<dyn TradingStrategy>,
}

impl MonteCarloSimulation {
    pub fn new(
        initial_pool: LiquidityPool,
        iterations: usize,
        steps_per_iteration: usize,
        strategy: Box<dyn TradingStrategy>,
    ) -> Self {
        Self {
            initial_pool,
            iterations,
            steps_per_iteration,
            strategy,
        }
    }

    /// Asynchronously runs a simulation based on the provided strategy, pool, and parameters specified in the struct instance.
    ///
    /// # Returns
    ///
    /// * `Result<SimulationResult, Box<dyn Error>>` - Returns a Result containing `SimulationResult` or an error.
    ///
    /// # SimulationResult
    ///
    /// * `average_price_change: Decimal` - The average change in price across all iterations.
    /// * `average_liquidity_change: Decimal` - The average change in liquidity across all iterations.
    /// * `max_price: Decimal` - The maximum observed price during the simulation.
    /// * `min_price: Decimal` - The minimum observed price during the simulation.
    ///
    /// # Errors
    ///
    /// If any of the asynchronous operations fail, an error boxed in `Box<dyn Error>` is returned.
    ///
    pub async fn run(&self) -> Result<SimulationResult, Box<dyn Error>> {
        if self.iterations == 0 {
            return Ok(SimulationResult {
                average_price_change: Decimal::ZERO,
                average_liquidity_change: Decimal::ZERO,
                max_price: Decimal::ZERO,
                min_price: Decimal::ZERO,
            });
        }

        let mut total_price_change = Decimal::ZERO;
        let mut total_liquidity_change = Decimal::ZERO;
        let mut max_price = Decimal::MIN;
        let mut min_price = Decimal::MAX;

        let results = stream::iter(0..self.iterations)
            .then(|_| {
                let pool = self.initial_pool.clone(); // Still need to clone here
                async move {
                    let mut pool = pool;
                    let initial_price = pool.get_price();
                    let initial_liquidity = pool.get_balances().0 + pool.get_balances().1;

                    for _ in 0..self.steps_per_iteration {
                        let current_price = pool.get_price();
                        if let Err(e) = self.strategy.execute(&mut pool, current_price).await {
                            error!("Strategy execution error: {}", e);
                        }
                    }

                    let final_price = pool.get_price();
                    let final_liquidity = pool.get_balances().0 + pool.get_balances().1;

                    (
                        (final_price - initial_price).abs(),
                        (final_liquidity - initial_liquidity).abs(),
                        final_price,
                    )
                }
            })
            .map(future::ready)
            .buffer_unordered(num_cpus::get())
            .collect::<Vec<_>>()
            .await;

        for (price_change, liquidity_change, price) in results {
            total_price_change += price_change;
            total_liquidity_change += liquidity_change;
            max_price = max_price.max(price);
            min_price = min_price.min(price);
        }

        Ok(SimulationResult {
            average_price_change: total_price_change / Decimal::from(self.iterations),
            average_liquidity_change: total_liquidity_change / Decimal::from(self.iterations),
            max_price,
            min_price,
        })
    }
}

/// `SimulationResult` is a data structure that holds the results of a simulation run.
/// It includes key metrics derived from the simulation such as average changes in price and liquidity,
/// and the highest and lowest prices encountered during the simulation.
///
/// Fields:
/// - `average_price_change`: A `Decimal` representing the average change in price throughout the simulation.
/// - `average_liquidity_change`: A `Decimal` representing the average change in liquidity throughout the simulation.
/// - `max_price`: A `Decimal` recording the highest price encountered during the simulation.
/// - `min_price`: A `Decimal` recording the lowest price encountered during the simulation.
pub struct SimulationResult {
    pub average_price_change: Decimal,
    pub average_liquidity_change: Decimal,
    pub max_price: Decimal,
    pub min_price: Decimal,
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
    simulation: &MonteCarloSimulation,
) -> Result<(SimulationResult, Duration), Box<dyn Error>> {
    let start = std::time::Instant::now();
    let result = simulation.run().await?;
    let duration = start.elapsed();
    Ok((result, duration))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use tokio;

    // Mock implementation of TradingStrategy
    struct MockTradingStrategy {}

    impl TradingStrategy for MockTradingStrategy {
        fn execute<'a>(
            &'a self,
            pool: &'a mut LiquidityPool,
            _: Decimal,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>> {
            Box::pin(async move {
                let amount_a = Decimal::new(10, 0);
                let amount_b = Decimal::new(5, 0);
                pool.add_liquidity(amount_a, amount_b)?;
                let swapped_b = pool.swap_a_to_b(amount_a)?;
                pool.swap_b_to_a(swapped_b)?;
                Ok(())
            })
        }
    }

    #[tokio::test]
    async fn test_monte_carlo_simulation() {
        let initial_pool = LiquidityPool::new(
            Decimal::new(1000, 0), // token_a
            Decimal::new(500, 0),  // token_b
            Decimal::new(1, 0),    // p_ref
            Decimal::new(1, 0),    // alpha
            Decimal::new(1, 0),    // beta
        );

        let strategy = Box::new(MockTradingStrategy {});
        let simulation = MonteCarloSimulation::new(initial_pool, 10, 5, strategy);
        let result = simulation.run().await.unwrap();

        assert!(result.average_price_change > Decimal::ZERO);
        assert!(result.average_liquidity_change > Decimal::ZERO);
        assert!(result.max_price > Decimal::ZERO);
        assert!(result.min_price > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_run_timed_simulation() {
        let initial_pool = LiquidityPool::new(
            Decimal::new(1000, 0), // token_a
            Decimal::new(500, 0),  // token_b
            Decimal::new(1, 0),    // p_ref
            Decimal::new(1, 0),    // alpha
            Decimal::new(1, 0),    // beta
        );

        let strategy = Box::new(MockTradingStrategy {});
        let simulation = MonteCarloSimulation::new(initial_pool, 10, 5, strategy);
        let (result, _duration) = run_timed_simulation(&simulation).await.unwrap();

        assert!(result.average_price_change > Decimal::ZERO);
        assert!(result.average_liquidity_change > Decimal::ZERO);
        assert!(result.max_price > Decimal::ZERO);
        assert!(result.min_price > Decimal::ZERO);
        // assert!(duration.as_secs() >= 0);
    }

    #[tokio::test]
    async fn test_monte_carlo_multiple_iterations() {
        let initial_pool = LiquidityPool::new(
            Decimal::new(1000, 0), // token_a
            Decimal::new(500, 0),  // token_b
            Decimal::new(1, 0),    // p_ref
            Decimal::new(1, 0),    // alpha
            Decimal::new(1, 0),    // beta
        );

        let strategy = Box::new(MockTradingStrategy {});
        let simulation = MonteCarloSimulation::new(initial_pool, 100, 50, strategy);
        let result = simulation.run().await.unwrap();

        assert!(result.average_price_change > Decimal::ZERO);
        assert!(result.average_liquidity_change > Decimal::ZERO);
        assert!(result.max_price > Decimal::ZERO);
        assert!(result.min_price > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_monte_carlo_with_low_liquidity() {
        let initial_pool = LiquidityPool::new(
            Decimal::new(1, 0), // token_a
            Decimal::new(1, 0), // token_b
            Decimal::new(1, 0), // p_ref
            Decimal::new(1, 0), // alpha
            Decimal::new(1, 0), // beta
        );

        let strategy = Box::new(MockTradingStrategy {});
        let simulation = MonteCarloSimulation::new(initial_pool, 10, 5, strategy);
        let result = simulation.run().await.unwrap();

        assert!(result.average_price_change >= Decimal::ZERO);
        assert!(result.average_liquidity_change >= Decimal::ZERO);
        assert!(result.max_price > Decimal::ZERO);
        assert!(result.min_price > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_monte_carlo_zero_iterations() {
        let initial_pool = LiquidityPool::new(
            Decimal::new(1000, 0), // token_a
            Decimal::new(500, 0),  // token_b
            Decimal::new(1, 0),    // p_ref
            Decimal::new(1, 0),    // alpha
            Decimal::new(1, 0),    // beta
        );

        let strategy = Box::new(MockTradingStrategy {});
        let simulation = MonteCarloSimulation::new(initial_pool, 0, 0, strategy);
        let result = simulation.run().await.unwrap();

        assert_eq!(result.average_price_change, Decimal::ZERO);
        assert_eq!(result.average_liquidity_change, Decimal::ZERO);
        assert_eq!(result.max_price, Decimal::ZERO);
        assert_eq!(result.min_price, Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_monte_carlo_high_iteration_count() {
        let initial_pool = LiquidityPool::new(
            Decimal::new(1000, 0), // token_a
            Decimal::new(500, 0),  // token_b
            Decimal::new(1, 0),    // p_ref
            Decimal::new(1, 0),    // alpha
            Decimal::new(1, 0),    // beta
        );

        let strategy = Box::new(MockTradingStrategy {});
        let simulation = MonteCarloSimulation::new(initial_pool, 10_000, 5, strategy);
        let result = simulation.run().await.unwrap();

        assert!(result.average_price_change > Decimal::ZERO);
        assert!(result.average_liquidity_change > Decimal::ZERO);
        assert!(result.max_price > Decimal::ZERO);
        assert!(result.min_price > Decimal::ZERO);
    }
}
