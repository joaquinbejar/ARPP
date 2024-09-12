/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use crate::arpp::liquidity_pool::LiquidityPool;
use crate::simulation::strategies::TradingStrategy;

use rust_decimal::Decimal;
use std::error::Error;
use rust_decimal_macros::dec;
use tracing::{debug, error, info};
use crate::analysis::metrics::{ analyze_simulation_results, PoolMetrics, PoolMetricsStep, accumulate_pool_metrics};
use crate::analysis::visualization::{create_price_chart, create_metrics_chart, create_simulation_analysis_chart};
use crate::arpp::formula::token_ratio;
use crate::simulation::result::{run_timed_simulation, SimulationResult};

pub struct MonteCarloSimulation {
    pool: LiquidityPool,
    iterations: usize,
    steps_per_iteration: usize,
    strategy: Box<dyn TradingStrategy>,
    price_history: Vec<Decimal>,
    reference_price_history: Vec<Decimal>,
    metrics_history: Vec<PoolMetrics>,
    alpha: Decimal,
    beta: Decimal,
}

impl MonteCarloSimulation {
    pub fn new(
        pool: LiquidityPool,
        iterations: usize,
        steps_per_iteration: usize,
        strategy: Box<dyn TradingStrategy>,
        alpha: Decimal,
        beta: Decimal,
    ) -> Self {
        Self {
            pool,
            iterations,
            steps_per_iteration,
            strategy,
            price_history: Vec::new(),
            reference_price_history: Vec::new(),
            metrics_history: Vec::new(),
            alpha,
            beta,
        }
    }

    /// Runs the Monte Carlo simulation with the given strategy.
    /// It modifies the same liquidity pool and adds liquidity if needed.
    ///
    /// # Returns
    /// `SimulationResult` with the average price change, liquidity change, max and min price.
    pub async fn run(&mut self) -> Result<SimulationResult, Box<dyn Error>> {
        if self.iterations == 0 {
            return Ok(SimulationResult::default());
        }

        let mut total_price_change = Decimal::ZERO;
        let mut total_liquidity_change = Decimal::ZERO;
        let mut max_price = Decimal::MIN;
        let mut min_price = Decimal::MAX;

        let (initial_a, initial_b) = self.pool.get_balances();
        let initial_price = self.pool.get_price();
        let initial_p_ref = self.pool.get_p_ref();
        let initial_ratio = token_ratio(initial_a, initial_b);

        let initial_step = PoolMetricsStep {
            price: initial_price,
            p_ref: initial_p_ref,
            balances_a: initial_a,
            balances_b: initial_b,
            ratio: initial_ratio,
        };

        let mut pool_metrics = PoolMetrics::new();

        for _ in 0..self.iterations {
            let initial_price = self.pool.get_price();
            let initial_liquidity = self.pool.get_balances().0 + self.pool.get_balances().1;

            for _ in 0..self.steps_per_iteration {
                let current_price = self.pool.get_price();
                self.pool.set_p_ref(self.alpha, self.beta); // set the reference price for this step

                accumulate_pool_metrics(&mut self.pool, &mut pool_metrics, &initial_step);

                self.add_liquidity_if_needed()?;

                if let Err(e) = self.strategy.execute(&mut self.pool, current_price).await {
                    error!("Strategy execution error: {}", e);
                }
            }

            let final_price = self.pool.get_price();
            let final_liquidity = self.pool.get_balances().0 + self.pool.get_balances().1;

            total_price_change += (final_price - initial_price).abs();
            total_liquidity_change += (final_liquidity - initial_liquidity).abs();
            max_price = max_price.max(final_price);
            min_price = min_price.min(final_price);
        }

        Ok(SimulationResult {
            average_price_change: total_price_change / Decimal::from(self.iterations),
            average_liquidity_change: total_liquidity_change / Decimal::from(self.iterations),
            max_price,
            min_price,
            metrics: pool_metrics,
        })
    }

    /// Adds liquidity to the pool if it falls below a certain threshold.
    fn add_liquidity_if_needed(&mut self) -> Result<(), Box<dyn Error>> {

        let min_liquidity = Decimal::new(1000, 0);  // Minimum liquidity for both currencies
        let token_a_liquidity = self.pool.get_balances().0;
        let token_b_liquidity = self.pool.get_balances().1;

        if token_a_liquidity < min_liquidity {
            let amount_a_to_add = min_liquidity - token_a_liquidity;
            self.pool.add_liquidity(amount_a_to_add, Decimal::new(10, 0))?;
            debug!("Adding liquidity to token A: {}", amount_a_to_add);
        }
        if token_b_liquidity < min_liquidity {
            let amount_b_to_add = min_liquidity - token_b_liquidity;
            self.pool.add_liquidity(Decimal::new(10, 0), amount_b_to_add)?;
            debug!("Adding liquidity to token B: {}", amount_b_to_add);
        }

        Ok(())
    }

    pub fn get_price_history(&self) -> Vec<Decimal> {
        self.price_history.clone()
    }

    pub fn get_metrics_history(&self) -> Vec<PoolMetrics> {
        self.metrics_history.clone()
    }

    pub fn get_final_pool(&self) -> LiquidityPool {
        self.pool.clone()
    }
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
    let alpha = Decimal::new(1, 1);
    let beta = Decimal::new(1, 2);
    let mut simulation = MonteCarloSimulation::new(initial_pool.clone(),
                                                   iterations,
                                                   steps,
                                                   strategy,
                                                   alpha,
                                                   beta);
    let (result, duration) = run_timed_simulation(&mut simulation).await?;

    info!("Simulation completed in {:?}", duration);
    info!("Average price change: {}", result.average_price_change);
    info!("Average liquidity change: {}", result.average_liquidity_change);
    info!("Maximum price: {}", result.max_price);
    info!("Minimum price: {}", result.min_price);

    // Calculate and display metrics
    info!("\nFinal Pool Metrics:");
    let pool_metrics = result.clone().metrics; //
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
    let alpha = dec!(0.2);
    let beta = dec!(5);
    create_price_chart(&simulation.get_price_history(), &result.metrics.get_p_ref(), "price_chart.png", alpha, beta)?;
    create_metrics_chart(&simulation.get_metrics_history(), "metrics_chart.png")?;
    create_simulation_analysis_chart(&analysis, "analysis_chart.png", alpha, beta)?;

    info!("Charts have been generated: price_chart.png, metrics_chart.png, analysis_chart.png");

    Ok(())
}

#[cfg(test)]
mod tests_monte_carlo {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use rust_decimal_macros::dec;
    use tokio;

    // Mock implementation of TradingStrategy
    struct MockTradingStrategy {}

    impl TradingStrategy for MockTradingStrategy {
        fn execute<'a>(
            &'a self,
            pool: &'a mut LiquidityPool,
            _: Decimal,
        ) -> Pin<Box<dyn Future<Output=Result<(), Box<dyn Error>>> + 'a>> {
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
        let mut simulation = MonteCarloSimulation::new(initial_pool,
                                                       10,
                                                       5,
                                                       strategy,
                                                       dec!(1),
                                                       dec!(1));
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
        let mut simulation = MonteCarloSimulation::new(initial_pool,
                                                       10,
                                                       5,
                                                       strategy,
                                                       dec!(1),
                                                       dec!(1));
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
        let mut simulation = MonteCarloSimulation::new(initial_pool,
                                                       100,
                                                       50,
                                                       strategy,
                                                       dec!(1),
                                                       dec!(1));
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
        let mut simulation = MonteCarloSimulation::new(initial_pool,
                                                       10,
                                                       5,
                                                       strategy,
                                                       dec!(1),
                                                       dec!(1));
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
        let mut simulation = MonteCarloSimulation::new(initial_pool,
                                                       0,
                                                       0,
                                                       strategy,
                                                       dec!(1),
                                                       dec!(1));
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
        let mut simulation = MonteCarloSimulation::new(initial_pool,
                                                       10_000,
                                                       5,
                                                       strategy,
                                                       dec!(1),
                                                       dec!(1));
        let result = simulation.run().await.unwrap();

        assert!(result.average_price_change > Decimal::ZERO);
        assert!(result.average_liquidity_change > Decimal::ZERO);
        assert!(result.max_price > Decimal::ZERO);
        assert!(result.min_price > Decimal::ZERO);
    }
}
