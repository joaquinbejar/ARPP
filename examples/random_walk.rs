/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 10/9/24
 ******************************************************************************/
use rust_decimal::Decimal;
use arpp::simulation::random_walk::{generate_multiple_random_walks, generate_random_walk_sequence};
use arpp::analysis::visualization::{visualize_random_walk, visualize_random_walks};
use arpp::utils::logger::setup_logger;

fn main() {
    setup_logger();
    let initial_price = Decimal::new(10000, 2);  // Initial price: 100.00
    let length = 30000;                            // Length of each sequence
    let std_dev = Decimal::new(1,1);          // Standard deviation: 0.1
    let std_dev_of_std_dev = Decimal::new(2, 2);  // Standard deviation of std_dev: 0.02
    let num_sequences = 20;                       // Number of sequences to generate

    // Generate multiple random walk sequences
    let sequences = generate_multiple_random_walks(num_sequences, initial_price, length, std_dev, std_dev_of_std_dev);


    let prices = sequences[0].clone();
    visualize_random_walk(prices, "draws/random_walk.png").unwrap();

    visualize_random_walks(sequences, "draws/random_walks.png").unwrap();

}
