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
use tracing::{error, info};
use crate::analysis::metrics::{calculate_pool_metrics, analyze_simulation_results, PoolMetrics};
use crate::analysis::visualization::{create_price_chart, create_metrics_chart, create_simulation_analysis_chart};

pub struct MonteCarloSimulation {
    initial_pool: LiquidityPool,
    iterations: usize,
    steps_per_iteration: usize,
    strategy: Box<dyn TradingStrategy>,
    price_history: Vec<Decimal>,
    metrics_history: Vec<PoolMetrics>,
    final_pool: Option<LiquidityPool>,
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
            price_history: Vec::new(),
            metrics_history: Vec::new(),
            final_pool: None,
        }
    }


    pub async fn run(&mut self) -> Result<SimulationResult, Box<dyn Error>> {
        if self.iterations == 0 {
            return Ok(SimulationResult::default());
        }

        let mut total_price_change = Decimal::ZERO;
        let mut total_liquidity_change = Decimal::ZERO;
        let mut max_price = Decimal::MIN;
        let mut min_price = Decimal::MAX;

        let initial_pool = self.initial_pool.clone();
        let iterations = self.iterations;
        let steps_per_iteration = self.steps_per_iteration;
        let strategy = &self.strategy;

        let results = stream::iter(0..iterations)
            .then(move |_| {
                let iteration_initial_pool = initial_pool.clone();
                async move {
                    let mut pool = iteration_initial_pool.clone();
                    let initial_price = pool.get_price();
                    let initial_liquidity = pool.get_balances().0 + pool.get_balances().1;
                    let mut iteration_price_history = Vec::new();
                    let mut iteration_metrics_history = Vec::new();

                    for _ in 0..steps_per_iteration {
                        let current_price = pool.get_price();
                        iteration_price_history.push(current_price);
                        iteration_metrics_history.push(calculate_pool_metrics(&pool, &iteration_initial_pool));

                        if let Err(e) = strategy.execute(&mut pool, current_price).await {
                            error!("Strategy execution error: {}", e);
                        }
                    }

                    let final_price = pool.get_price();
                    let final_liquidity = pool.get_balances().0 + pool.get_balances().1;

                    (
                        (final_price - initial_price).abs(),
                        (final_liquidity - initial_liquidity).abs(),
                        final_price,
                        iteration_price_history,
                        iteration_metrics_history,
                        pool,
                    )
                }
            })
            .map(|future_result| future::ready(future_result))
            .buffer_unordered(num_cpus::get())
            .collect::<Vec<_>>()
            .await;

        let mut all_prices = Vec::new();
        let mut all_metrics = Vec::new();

        for (price_change, liquidity_change, price, prices, metrics, pool) in results {
            total_price_change += price_change;
            total_liquidity_change += liquidity_change;
            max_price = max_price.max(price);
            min_price = min_price.min(price);
            all_prices.extend(prices);
            all_metrics.extend(metrics);
            self.final_pool = Some(pool);  // Store the last pool state
        }

        self.price_history = all_prices;
        self.metrics_history = all_metrics;

        Ok(SimulationResult {
            average_price_change: total_price_change / Decimal::from(iterations),
            average_liquidity_change: total_liquidity_change / Decimal::from(iterations),
            max_price,
            min_price,
        })
    }


    pub fn get_price_history(&self) -> Vec<Decimal> {
        self.price_history.clone()
    }

    pub fn get_metrics_history(&self) -> Vec<PoolMetrics> {
        self.metrics_history.clone()
    }

    pub fn get_final_pool(&self) -> LiquidityPool {
        self.final_pool.clone().expect("Simulation has not been run yet")
    }
}


#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub average_price_change: Decimal,
    pub average_liquidity_change: Decimal,
    pub max_price: Decimal,
    pub min_price: Decimal,
}

impl Default for SimulationResult {
    fn default() -> Self {
        SimulationResult {
            average_price_change: Decimal::ZERO,
            average_liquidity_change: Decimal::ZERO,
            max_price: Decimal::ZERO,
            min_price: Decimal::ZERO,
        }
    }
}

impl SimulationResult {
    pub fn new(
        average_price_change: Decimal,
        average_liquidity_change: Decimal,
        max_price: Decimal,
        min_price: Decimal,
    ) -> Self {
        Self {
            average_price_change,
            average_liquidity_change,
            max_price,
            min_price,
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

#[allow(dead_code)]
async fn run_monte_carlo(
    strategy: Box<dyn TradingStrategy>,
    iterations: usize,
    steps: usize,
    initial_token_a: Decimal,
    initial_token_b: Decimal,
) -> Result<(), Box<dyn Error>> {
    let initial_pool = LiquidityPool::new(
        initial_token_a,
        initial_token_b,
        Decimal::ONE,  // p_ref
        Decimal::new(5, 1),  // alpha (0.5)
        Decimal::ONE,  // beta
    );

    let mut simulation = MonteCarloSimulation::new(initial_pool.clone(), iterations, steps, strategy);
    let (result, duration) = run_timed_simulation(&mut simulation).await?;

    info!("Simulation completed in {:?}", duration);
    info!("Average price change: {}", result.average_price_change);
    info!("Average liquidity change: {}", result.average_liquidity_change);
    info!("Maximum price: {}", result.max_price);
    info!("Minimum price: {}", result.min_price);

    // Calculate and display metrics
    let final_pool = simulation.get_final_pool();
    let pool_metrics = calculate_pool_metrics(&final_pool, &initial_pool);
    info!("Pool Metrics:");
    info!("Price Volatility: {}", pool_metrics.price_volatility);
    info!("Liquidity Depth: {}", pool_metrics.liquidity_depth);
    info!("Trading Volume: {}", pool_metrics.trading_volume);
    info!("Impermanent Loss: {}", pool_metrics.impermanent_loss);

    let analysis = analyze_simulation_results(&result);
    info!("Simulation Analysis:");
    info!("Price Stability: {}", analysis.price_stability);
    info!("Average Price Impact: {}", analysis.average_price_impact);
    info!("Liquidity Efficiency: {}", analysis.liquidity_efficiency);

    // Generate visualizations
    create_price_chart(&simulation.get_price_history(), "price_chart.png")?;
    create_metrics_chart(&simulation.get_metrics_history(), "metrics_chart.png")?;
    create_simulation_analysis_chart(&analysis, "analysis_chart.png")?;

    info!("Charts have been generated: price_chart.png, metrics_chart.png, analysis_chart.png");

    Ok(())
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
        let mut simulation = MonteCarloSimulation::new(initial_pool, 10, 5, strategy);
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
        let mut simulation = MonteCarloSimulation::new(initial_pool, 10, 5, strategy);
        let (result, _duration) = run_timed_simulation(&mut simulation).await.unwrap();

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
        let mut simulation = MonteCarloSimulation::new(initial_pool, 100, 50, strategy);
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
        let mut simulation = MonteCarloSimulation::new(initial_pool, 10, 5, strategy);
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
        let mut simulation = MonteCarloSimulation::new(initial_pool, 0, 0, strategy);
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
        let mut simulation = MonteCarloSimulation::new(initial_pool, 10_000, 5, strategy);
        let result = simulation.run().await.unwrap();

        assert!(result.average_price_change > Decimal::ZERO);
        assert!(result.average_liquidity_change > Decimal::ZERO);
        assert!(result.max_price > Decimal::ZERO);
        assert!(result.min_price > Decimal::ZERO);
    }
}
