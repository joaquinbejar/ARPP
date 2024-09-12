/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use crate::arpp::liquidity_pool::LiquidityPool;
use crate::simulation::monte_carlo::MonteCarloSimulation;
use crate::simulation::result::run_timed_simulation;
use crate::simulation::strategies::{MeanReversionStrategy, RandomStrategy, TradingStrategy};
use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::info;

/**
 * The `SimulationCommand` enum represents the different subcommands that can be used to run a simulation.

cargo run -- simulate mean-reversion --iterations 1000 --steps 100 --target-price 1.5 --swap-threshold 0.05
cargo run -- simulate random --iterations 1000 --steps 100 --swap-probability 0.6

 */

#[derive(Subcommand)]
pub enum SimulationCommand {
    /// Run a Monte Carlo simulation with a random trading strategy
    Random(RandomSimulationArgs),
    /// Run a Monte Carlo simulation with a mean reversion trading strategy
    MeanReversion(MeanReversionSimulationArgs),
}

/// `RandomSimulationArgs` is a struct used to define the arguments for a random simulation.
///
/// # Arguments
///
/// * `iterations` - The number of iterations to perform in the simulation. Default value is 1000.
/// * `steps` - The number of steps to take in each iteration. Default value is 100.
/// * `swap_probability` - The probability of a swap occurring at each step. Default value is 0.5.
/// * `max_swap_amount` - The maximum amount that can be swapped during a simulation step. Default value is 10.
/// * `initial_token_a` - The initial amount of token A. Default value is 1000.
/// * `initial_token_b` - The initial amount of token B. Default value is 1000.
///
/// The `Args` derive macro is used to parse command line arguments based on the struct definition.
#[derive(Args)]
pub struct RandomSimulationArgs {
    #[arg(long, default_value = "1000")]
    iterations: usize,
    #[arg(long, default_value = "100")]
    steps: usize,
    #[arg(long, default_value = "0.5")]
    swap_probability: f64,
    #[arg(long, default_value = "10")]
    max_swap_amount: Decimal,
    #[arg(long, default_value = "1000")]
    initial_token_a: Decimal,
    #[arg(long, default_value = "1000")]
    initial_token_b: Decimal,
}

/// Struct representing the arguments for mean reversion simulation.
///
/// This struct holds various parameters that can be configured for
/// running a mean reversion simulation through command-line arguments.
/// Each field is represented with a corresponding command-line argument.
///
/// # Fields:
/// - `iterations`: The number of iterations to run for the simulation (default: 1000).
/// - `steps`: The number of steps to simulate in each iteration (default: 100).
/// - `target_price`: The target price for the mean reversion (default: 1).
/// - `swap_threshold`: The price threshold to trigger a swap (default: 0.1).
/// - `swap_amount`: The amount to swap when the threshold is breached (default: 10).
/// - `initial_token_a`: The initial amount of token A for the simulation (default: 1000).
/// - `initial_token_b`: The initial amount of token B for the simulation (default: 1000).
#[derive(Args)]
pub struct MeanReversionSimulationArgs {
    #[arg(long, default_value = "1000")]
    iterations: usize,
    #[arg(long, default_value = "100")]
    steps: usize,
    #[arg(long, default_value = "1")]
    target_price: Decimal,
    #[arg(long, default_value = "0.1")]
    swap_threshold: Decimal,
    #[arg(long, default_value = "10")]
    swap_amount: Decimal,
    #[arg(long, default_value = "1000")]
    initial_token_a: Decimal,
    #[arg(long, default_value = "1000")]
    initial_token_b: Decimal,
}

/// Asynchronously runs a simulation based on the provided simulation command.
///
/// # Arguments
///
/// * `cmd` - A reference to a `SimulationCommand` enum which specifies the type and parameters of the simulation.
///
/// # Returns
///
/// A Result with an empty tuple `Result<(), Box<dyn Error>>` that signifies the completion state of the function.
///
/// # Errors
///
/// Returns an error if the simulation fails to execute.
///
pub async fn run_simulation(cmd: &SimulationCommand) -> Result<(), Box<dyn Error>> {
    match cmd {
        SimulationCommand::Random(args) => {
            let strategy = Box::new(RandomStrategy::new(
                args.swap_probability,
                args.max_swap_amount,
            ));
            run_monte_carlo(
                strategy,
                args.iterations,
                args.steps,
                args.initial_token_a,
                args.initial_token_b,
            )
            .await
        }
        SimulationCommand::MeanReversion(args) => {
            let strategy = Box::new(MeanReversionStrategy::new(
                args.swap_threshold,
                args.swap_amount,
            ));
            run_monte_carlo(
                strategy,
                args.iterations,
                args.steps,
                args.initial_token_a,
                args.initial_token_b,
            )
            .await
        }
    }
}

/// Runs a Monte Carlo simulation for a given trading strategy.
///
/// This asynchronous function sets up and executes a Monte Carlo simulation
/// to evaluate the performance of a provided trading strategy over a series of iterations and steps.
///
/// # Arguments
///
/// * `strategy` - A boxed dynamic trading strategy implementing the `TradingStrategy` trait.
/// * `iterations` - The number of iterations to perform in the simulation.
/// * `steps` - The number of steps to perform in each iteration of the simulation.
/// * `initial_token_a` - The initial amount of token A in the liquidity pool.
/// * `initial_token_b` - The initial amount of token B in the liquidity pool.
///
/// # Returns
///
/// This function returns a `Result` containing `Ok(())` if the simulation completed successfully,
/// or an error wrapped in a boxed `Error` trait object if any occurred.
///
/// # Errors
///
/// This function will return an error if any of the following scenarios occur:
/// - The `run_timed_simulation` function fails to execute or returns an error.
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
        Decimal::ONE,       // p_ref
        Decimal::new(5, 1), // alpha (0.5)
        Decimal::ONE,       // beta
    );

    let mut simulation = MonteCarloSimulation::new(
        initial_pool,
        iterations,
        steps,
        strategy,
        Decimal::ONE,
        Decimal::ONE,
    );
    let (result, duration) = run_timed_simulation(&mut simulation).await?;

    info!("Simulation completed in {:?}", duration);
    info!("Average price change: {}", result.average_price_change);
    info!(
        "Average liquidity change: {}",
        result.average_liquidity_change
    );
    info!("Maximum price: {}", result.max_price);
    info!("Minimum price: {}", result.min_price);

    Ok(())
}

#[cfg(test)]
mod tests_commands {
    use super::*;
    use rust_decimal::prelude::Decimal;
    use std::error::Error;
    use std::future::Future;
    use std::pin::Pin;
    use tokio::runtime::Runtime;

    /// Dummy TradingStrategy for testing
    struct DummyStrategy;

    impl TradingStrategy for DummyStrategy {
        fn execute<'a>(
            &'a self,
            _pool: &'a mut LiquidityPool,
            _current_price: Decimal,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>> {
            Box::pin(async { Ok(()) })
        }
    }

    #[tokio::test]
    async fn test_random_simulation_default_args() -> Result<(), Box<dyn Error>> {
        let args = RandomSimulationArgs {
            iterations: 1000,
            steps: 100,
            swap_probability: 0.5,
            max_swap_amount: Decimal::new(10, 0),
            initial_token_a: Decimal::new(1000, 0),
            initial_token_b: Decimal::new(1000, 0),
        };
        let cmd = SimulationCommand::Random(args);
        let result = run_simulation(&cmd).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_mean_reversion_simulation_default_args() -> Result<(), Box<dyn Error>> {
        let args = MeanReversionSimulationArgs {
            iterations: 1000,
            steps: 100,
            target_price: Decimal::ONE,
            swap_threshold: Decimal::new(1, 1), // 0.1
            swap_amount: Decimal::new(10, 0),
            initial_token_a: Decimal::new(1000, 0),
            initial_token_b: Decimal::new(1000, 0),
        };
        let cmd = SimulationCommand::MeanReversion(args);
        let result = run_simulation(&cmd).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_random_strategy_execution() -> Result<(), Box<dyn Error>> {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let strategy = Box::new(DummyStrategy);
            let result = run_monte_carlo(
                strategy,
                100,
                10,
                Decimal::new(1000, 0),
                Decimal::new(1000, 0),
            )
            .await;
            assert!(result.is_ok());
            Ok(())
        })
    }

    #[test]
    fn test_mean_reversion_strategy_execution() -> Result<(), Box<dyn Error>> {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let strategy = Box::new(DummyStrategy);
            let result = run_monte_carlo(
                strategy,
                100,
                10,
                Decimal::new(1000, 0),
                Decimal::new(1000, 0),
            )
            .await;
            assert!(result.is_ok());
            Ok(())
        })
    }
}
