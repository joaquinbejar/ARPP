/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

/// This Rust program is an example of a Command Line Interface (CLI) application
/// that uses the `clap` crate for parsing command line arguments and the `tokio`
/// crate for asynchronous runtime.
///
/// ## Components:
///
/// - `Cli`: This structure holds the parsed command line arguments. It uses the
///   `clap::Parser` derive macro to automatically generate the parsing logic.
/// - `Commands`: This enum contains the different subcommands available in the CLI.
///   Currently, it includes a `Simulate` command which holds a nested subcommand
///   named `simulation` of type `SimulationCommand`.
/// - `main` function: This is the entry point of the application. It first parses
///   the CLI arguments, sets up logging using `setup_logger`, and then matches
///   on the parsed command to execute the corresponding action, in this case,
///   running a simulation.
///
/// ## Usage:
///
/// Compile the program and run it from the command line with appropriate subcommands.
///
/// ```sh
/// $ cargo build --release
/// $ ./target/release/my_program simulate <simulation_subcommand>
/// ```
///
/// All subcommands and their arguments will be shown if used improperly.
///
///  The `SimulationCommand` enum represents the different subcommands that can be used to run a simulation.
///
/// cargo run -- simulate mean-reversion --iterations 1000 --steps 100 --target-price 1.5 --swap-threshold 0.05
/// cargo run -- simulate random --iterations 1000 --steps 100 --swap-probability 0.6
///
///
use arpp::cli::commands::{run_simulation, SimulationCommand};
use arpp::utils::logger::setup_logger;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Simulate {
        #[command(subcommand)]
        simulation: SimulationCommand,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    setup_logger();

    match &cli.command {
        Commands::Simulate { simulation } => {
            run_simulation(simulation).await?;
        }
    }

    Ok(())
}