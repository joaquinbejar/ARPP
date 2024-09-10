/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use crate::arpp::liquidity_pool::LiquidityPool;
use crate::simulation::strategies::{RandomStrategy, MeanReversionStrategy, TradingStrategy};
use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::info;
use crate::simulation::monte_carlo::MonteCarloSimulation;
use crate::simulation::result::run_timed_simulation;

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

pub async fn run_simulation(cmd: &SimulationCommand) -> Result<(), Box<dyn Error>> {
    match cmd {
        SimulationCommand::Random(args) => {
            let strategy = Box::new(RandomStrategy::new(args.swap_probability, args.max_swap_amount));
            run_monte_carlo(
                strategy,
                args.iterations,
                args.steps,
                args.initial_token_a,
                args.initial_token_b,
            ).await
        },
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
            ).await
        },
    }
}

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

    let mut simulation = MonteCarloSimulation::new(initial_pool, iterations, steps, strategy, Decimal::ONE, Decimal::ONE);
    let (result, duration) = run_timed_simulation(&mut simulation).await?;

    info!("Simulation completed in {:?}", duration);
    info!("Average price change: {}", result.average_price_change);
    info!("Average liquidity change: {}", result.average_liquidity_change);
    info!("Maximum price: {}", result.max_price);
    info!("Minimum price: {}", result.min_price);

    Ok(())
}