/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/
use rand::prelude::*;
use rand_distr::{Distribution, Normal};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

const MIN_PRICE: f64 = 0.1;
/// Generates a new price based on a random walk model.
///
/// This function simulates the change in price using a random walk model. The randomness
/// is influenced by the standard deviation (`std_dev`) and the standard deviation
/// of the standard deviation (`std_dev_of_std_dev`). It ensures that the resulting
/// price change follows a normal distribution.
///
/// # Parameters
///
/// * `last_price` - The starting price before applying the random walk.
/// * `std_dev` - The standard deviation used for the normal distribution of price changes.
/// * `std_dev_of_std_dev` - The standard deviation of the standard deviation, providing
///   a variability to the standard deviation itself.
///
/// # Returns
///
/// * A `Decimal` representing the new price after applying the random walk.
///
pub fn random_walk_price(
    last_price: Decimal,
    std_dev: Decimal,
    std_dev_of_std_dev: Decimal,
) -> Decimal {
    let mut rng = thread_rng();

    let std_dev_f64 = std_dev.to_f64().unwrap();
    let std_dev_of_std_dev_f64 = std_dev_of_std_dev.to_f64().unwrap();

    let std_dev_dist = Normal::new(std_dev_f64, std_dev_of_std_dev_f64).unwrap();
    let new_std_dev = Decimal::from_f64(std_dev_dist.sample(&mut rng).abs()).unwrap();

    let price_change_dist = Normal::new(0.0, new_std_dev.to_f64().unwrap()).unwrap();
    let price_change = Decimal::from_f64(price_change_dist.sample(&mut rng)).unwrap();

    let new_price = last_price + price_change;
    new_price.max(Decimal::from_f64(MIN_PRICE).unwrap())
}

/// Generates a vector of prices based on a random walk.
///
/// # Arguments:
///
/// * `initial_price` - The starting price for the random walk.
/// * `length` - The number of prices to generate (size of the resulting vector).
/// * `std_dev` - Initial standard deviation for the price changes.
/// * `std_dev_of_std_dev` - Standard deviation of the standard deviation (variability of the std deviation).
///
/// # Returns:
///
/// A vector of `Decimal` prices representing the random walk sequence.
pub fn generate_random_walk_sequence(
    initial_price: Decimal,
    length: usize,
    std_dev: Decimal,
    std_dev_of_std_dev: Decimal,
) -> Vec<Decimal> {
    // Initialize the vector with the first price
    let mut prices = Vec::with_capacity(length);
    let mut current_price = initial_price;

    // Add the initial price to the vector
    prices.push(current_price);

    // Generate the rest of the prices in the sequence
    for _ in 1..length {
        // Calculate the next price using the random walk
        current_price = random_walk_price(current_price, std_dev, std_dev_of_std_dev);
        // Add the new price to the vector
        prices.push(current_price);
    }

    prices
}

/// Generates a vector of random walk sequences.
///
/// # Arguments:
///
/// * `num_sequences` - The number of random walk sequences to generate.
/// * `initial_price` - The starting price for each random walk.
/// * `length` - The number of prices in each sequence (size of each sequence).
/// * `std_dev` - Initial standard deviation for the price changes.
/// * `std_dev_of_std_dev` - Standard deviation of the standard deviation (variability of the std deviation).
///
/// # Returns:
///
/// A vector of vectors, where each inner vector represents a random walk sequence.
pub fn generate_multiple_random_walks(
    num_sequences: usize,
    initial_price: Decimal,
    length: usize,
    std_dev: Decimal,
    std_dev_of_std_dev: Decimal,
) -> Vec<Vec<Decimal>> {
    // Create a vector of sequences
    let mut sequences = Vec::with_capacity(num_sequences);

    // Generate each sequence
    for _ in 0..num_sequences {
        let sequence =
            generate_random_walk_sequence(initial_price, length, std_dev, std_dev_of_std_dev);
        sequences.push(sequence);
    }

    sequences
}

#[cfg(test)]
mod tests_random_walk_price {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_random_walk_price_increases() {
        let last_price = Decimal::new(10000, 2); // 100.00
        let std_dev = Decimal::new(100, 2); // 1.00
        let std_dev_of_std_dev = Decimal::new(20, 2); // 0.20
        let new_price = random_walk_price(last_price, std_dev, std_dev_of_std_dev);
        assert!(new_price >= Decimal::ZERO);
    }

    #[test]
    fn test_random_walk_price_changes() {
        let last_price = Decimal::new(10000, 2); // 100.00
        let std_dev = Decimal::new(100, 2); // 1.00
        let std_dev_of_std_dev = Decimal::new(20, 2); // 0.20
        let new_price = random_walk_price(last_price, std_dev, std_dev_of_std_dev);
        assert!(new_price != last_price);
    }

    #[test]
    fn test_random_walk_with_zero_std_dev() {
        let last_price = Decimal::new(10000, 2);
        let std_dev = Decimal::ZERO;
        let std_dev_of_std_dev = Decimal::ZERO;
        let new_price = random_walk_price(last_price, std_dev, std_dev_of_std_dev);
        assert_eq!(new_price, last_price);
    }

    #[test]
    fn test_random_walk_with_high_std_dev() {
        let last_price = Decimal::new(10000, 2);
        let std_dev = Decimal::new(5000, 2);
        let std_dev_of_std_dev = Decimal::new(500, 2);
        let new_price = random_walk_price(last_price, std_dev, std_dev_of_std_dev);
        assert!(new_price != last_price);
    }

    #[test]
    fn test_random_walk_with_minimal_values() {
        let last_price = Decimal::new(1, 0);
        let std_dev = Decimal::new(1, 2);
        let std_dev_of_std_dev = Decimal::new(1, 3);
        let new_price = random_walk_price(last_price, std_dev, std_dev_of_std_dev);
        assert!(new_price >= Decimal::ZERO);
    }
}
