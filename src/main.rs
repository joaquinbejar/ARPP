/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use clap::{Parser, Subcommand};
use arpp::cli::commands::{run_simulation, SimulationCommand};
use arpp::utils::logger::setup_logger;

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
        },
    }

    Ok(())
}