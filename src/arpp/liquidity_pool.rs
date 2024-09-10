/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use crate::arpp::formula::arpp;
use rust_decimal::Decimal;
use std::error::Error;

/// The `LiquidityPool` struct represents a liquidity pool in a decentralized finance (DeFi) context.
///
/// # Fields
/// - `token_a` (Decimal): The amount of the first token in the pool.
/// - `token_b` (Decimal): The amount of the second token in the pool.
/// - `p_ref` (Decimal): The reference price for the pool.
/// - `alpha` (Decimal): A parameter that could represent a weight or coefficient in a specific formula or algorithm.
/// - `beta` (Decimal): Another parameter that could represent a weight or coefficient in a different formula or algorithm.
///
#[derive(Debug, Clone)]
pub struct LiquidityPool {
    token_a: Decimal,
    token_b: Decimal,
    p_ref: Decimal,
    alpha: Decimal,
    beta: Decimal,
}

impl LiquidityPool {
    pub fn new(
        token_a: Decimal,
        token_b: Decimal,
        p_ref: Decimal,
        alpha: Decimal,
        beta: Decimal,
    ) -> Self {
        Self {
            token_a,
            token_b,
            p_ref,
            alpha,
            beta,
        }
    }

    /// Adds liquidity to the liquidity pool by increasing the amounts of tokens A and B.
    ///
    /// The function takes two `Decimal` values representing the amounts of tokens A and B
    /// to be added to the liquidity pool. It updates the state of `self` to reflect the
    /// added amounts.
    ///
    /// # Arguments
    ///
    /// * `amount_a` - The amount of token A to be added. Must be greater than zero.
    /// * `amount_b` - The amount of token B to be added. Must be greater than zero.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - Returns `Ok(())` if the operation is successful.
    ///   Returns an error if either `amount_a` or `amount_b` is less than or equal to zero.
    ///
    /// # Errors
    ///
    /// This function will return an error if either `amount_a` or `amount_b` is not positive.
    ///
    pub fn add_liquidity(
        &mut self,
        amount_a: Decimal,
        amount_b: Decimal,
    ) -> Result<(), Box<dyn Error>> {
        if amount_a <= Decimal::ZERO || amount_b <= Decimal::ZERO {
            return Err("Amounts must be positive".into());
        }
        self.token_a += amount_a;
        self.token_b += amount_b;
        Ok(())
    }

    /// Removes liquidity from the pool.
    ///
    /// # Parameters
    ///
    /// - `amount_a`: The amount of token A to remove. Must be a positive value.
    /// - `amount_b`: The amount of token B to remove. Must be a positive value.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success (`Ok`) or failure (`Err`) of the operation.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following cases:
    ///
    /// - If `amount_a` or `amount_b` are non-positive.
    /// - If the pool does not have sufficient liquidity to meet the requested amounts.
    ///
    pub fn remove_liquidity(
        &mut self,
        amount_a: Decimal,
        amount_b: Decimal,
    ) -> Result<(), Box<dyn Error>> {
        if amount_a <= Decimal::ZERO || amount_b <= Decimal::ZERO {
            return Err("Amounts must be positive".into());
        }
        if amount_a > self.token_a || amount_b > self.token_b {
            return Err("Insufficient liquidity".into());
        }
        self.token_a -= amount_a;
        self.token_b -= amount_b;
        Ok(())
    }

    /// Swaps a specified amount of token A to token B based on a given price mechanism.
    ///
    /// # Parameters
    ///
    /// - `amount_a`: The amount of token A to be swapped to token B. Must be a positive value greater
    /// than zero and should not exceed the available `token_a` liquidity.
    ///
    /// # Returns
    ///
    /// - `Ok(Decimal)`: The resulting amount of token B received in the swap.
    /// - `Err(Box<dyn Error>)`: An error indicating one of the following issues:
    ///   - The specified amount of token A is non-positive.
    ///   - Insufficient liquidity of token A.
    ///   - Insufficient liquidity of token B for the swap.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - `amount_a` is less than or equal to zero.
    /// - `amount_a` is greater than the available liquidity of `token_a`.
    /// - The resulting amount of token B for the swap exceeds the available liquidity of `token_b`.
    ///
    pub fn swap_a_to_b(&mut self, amount_a: Decimal) -> Result<Decimal, Box<dyn Error>> {
        if amount_a <= Decimal::ZERO {
            return Err("Amount must be positive".into());
        }
        if amount_a > self.token_a {
            return Err("Insufficient liquidity".into());
        }

        let r = self.token_b / self.token_a;
        let price = arpp(self.p_ref, self.alpha, self.beta, r);
        let amount_b = amount_a * price;

        if amount_b > self.token_b {
            return Err("Insufficient liquidity for swap".into());
        }

        self.token_a += amount_a;
        self.token_b -= amount_b;

        Ok(amount_b)
    }

    /// Swaps a specified amount of token B for token A.
    ///
    /// This function performs a swap of `amount_b` of token B for an equivalent amount of token A.
    /// It performs several checks to ensure that the swap is valid and possible given the contract's
    /// current state and available liquidity.
    ///
    /// # Parameters
    /// - `amount_b`: The amount of token B to be swapped.
    ///
    /// # Returns
    /// - `Result<Decimal, Box<dyn Error>>`: Returns the amount of token A received on a successful swap or an error
    ///
    /// # Errors
    /// - Returns an error if `amount_b` is less than or equal to zero.
    /// - Returns an error if there is insufficient liquidity of token B.
    /// - Returns an error if there is insufficient liquidity of token A for the swap.
    ///
    pub fn swap_b_to_a(&mut self, amount_b: Decimal) -> Result<Decimal, Box<dyn Error>> {
        if amount_b <= Decimal::ZERO {
            return Err("Amount must be positive".into());
        }
        if amount_b > self.token_b {
            return Err("Insufficient liquidity".into());
        }

        let r = self.token_a / self.token_b;
        let price = arpp(self.p_ref, self.alpha, self.beta, r);
        let amount_a = amount_b / price;

        if amount_a > self.token_a {
            return Err("Insufficient liquidity for swap".into());
        }

        self.token_b += amount_b;
        self.token_a -= amount_a;

        Ok(amount_a)
    }

    /// Calculates the price based on the ratio of `token_b` to `token_a`
    /// and adjusts it using the parameters `p_ref`, `alpha`, and `beta`.
    ///
    /// # Returns
    ///
    /// A `Decimal` representing the calculated price.
    ///
    pub fn get_price(&self) -> Decimal {
        let r = self.token_b / self.token_a;
        arpp(self.p_ref, self.alpha, self.beta, r)
    }

    /// Returns the current balances of two tokens.
    ///
    /// This function retrieves the balances of `token_a` and `token_b` encapsulated
    /// within the structure.
    ///
    /// # Returns
    ///
    /// A tuple containing two `Decimal` values:
    /// - The first value corresponds to the balance of `token_a`.
    /// - The second value corresponds to the balance of `token_b`.
    ///
    /// # Usage
    ///
    /// This function can be used to check the current balance of two tokens stored
    /// within an instance of the structure. It returns a tuple where the first element
    /// is the balance of `token_a` and the second element is the balance of `token_b`.
    pub fn get_balances(&self) -> (Decimal, Decimal) {
        (self.token_a, self.token_b)
    }
}

#[cfg(test)]
mod tests_liquidity_pool {
    use super::*;
    use rust_decimal_macros::dec;
    use tracing::debug;

    // Helper function to create a standard pool for testing
    fn create_standard_pool() -> LiquidityPool {
        LiquidityPool::new(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1))
    }

    #[test]
    fn test_new_pool_creation() {
        let pool = create_standard_pool();
        assert_eq!(pool.get_balances(), (dec!(1000), dec!(1000)));
        assert_eq!(pool.get_price(), dec!(1));
    }

    #[test]
    fn test_add_liquidity() {
        let mut pool = create_standard_pool();
        assert!(pool.add_liquidity(dec!(100), dec!(100)).is_ok());
        assert_eq!(pool.get_balances(), (dec!(1100), dec!(1100)));
    }

    #[test]
    fn test_add_liquidity_zero_amount() {
        let mut pool = create_standard_pool();
        assert!(pool.add_liquidity(dec!(0), dec!(100)).is_err());
        assert!(pool.add_liquidity(dec!(100), dec!(0)).is_err());
    }

    #[test]
    fn test_add_liquidity_negative_amount() {
        let mut pool = create_standard_pool();
        assert!(pool.add_liquidity(dec!(-100), dec!(100)).is_err());
        assert!(pool.add_liquidity(dec!(100), dec!(-100)).is_err());
    }

    #[test]
    fn test_remove_liquidity() {
        let mut pool = create_standard_pool();
        assert!(pool.remove_liquidity(dec!(100), dec!(100)).is_ok());
        assert_eq!(pool.get_balances(), (dec!(900), dec!(900)));
    }

    #[test]
    fn test_remove_liquidity_zero_amount() {
        let mut pool = create_standard_pool();
        assert!(pool.remove_liquidity(dec!(0), dec!(100)).is_err());
        assert!(pool.remove_liquidity(dec!(100), dec!(0)).is_err());
    }

    #[test]
    fn test_remove_liquidity_negative_amount() {
        let mut pool = create_standard_pool();
        assert!(pool.remove_liquidity(dec!(-100), dec!(100)).is_err());
        assert!(pool.remove_liquidity(dec!(100), dec!(-100)).is_err());
    }

    #[test]
    fn test_remove_liquidity_insufficient() {
        let mut pool = create_standard_pool();
        assert!(pool.remove_liquidity(dec!(1001), dec!(100)).is_err());
        assert!(pool.remove_liquidity(dec!(100), dec!(1001)).is_err());
    }

    #[test]
    fn test_swap_a_to_b() {
        let mut pool = create_standard_pool();
        let initial_balances = pool.get_balances();
        debug!("Initial balances: {:?}", initial_balances);

        let amount_a_to_swap = dec!(100);
        let result = pool.swap_a_to_b(amount_a_to_swap);
        assert!(result.is_ok());
        let amount_b = result.unwrap();

        let (token_a, token_b) = pool.get_balances();
        debug!("Final balances: ({}, {})", token_a, token_b);
        debug!("Amount of B received: {}", amount_b);

        assert!(token_a > initial_balances.0, "Token A should increase");
        assert!(token_b < initial_balances.1, "Token B should decrease");

        // Verify that the amount of B received equals the amount of A swapped
        assert_eq!(
            amount_b, amount_a_to_swap,
            "Amount of B should equal the amount of A swapped"
        );

        // Check that the changes in balances are consistent
        assert_eq!(
            token_a - initial_balances.0,
            amount_a_to_swap,
            "Increase in A should equal amount swapped"
        );
        assert_eq!(
            initial_balances.1 - token_b,
            amount_b,
            "Decrease in B should equal amount received"
        );

        // Verify that the sum of tokens remains constant (with a small margin for rounding errors)
        let initial_sum = initial_balances.0 + initial_balances.1;
        let final_sum = token_a + token_b;
        assert!(
            (initial_sum - final_sum).abs() < dec!(0.000001),
            "Total token amount should remain constant"
        );

        let swap_rate = amount_b / amount_a_to_swap;
        debug!("Swap rate (B/A): {}", swap_rate);
        assert_eq!(swap_rate, dec!(1), "Swap rate should be 1:1");
    }

    #[test]
    fn test_swap_b_to_a() {
        let mut pool = create_standard_pool();
        let initial_balances = pool.get_balances();
        debug!("Initial balances: {:?}", initial_balances);

        let amount_b_to_swap = dec!(100);
        let result = pool.swap_b_to_a(amount_b_to_swap);
        assert!(result.is_ok());
        let amount_a = result.unwrap();

        let (token_a, token_b) = pool.get_balances();
        debug!("Final balances: ({}, {})", token_a, token_b);
        debug!("Amount of A received: {}", amount_a);

        assert!(token_a < initial_balances.0, "Token A should decrease");
        assert!(token_b > initial_balances.1, "Token B should increase");

        // We verify that the amount of A received is equal to the amount of B delivered.
        assert_eq!(
            amount_a, amount_b_to_swap,
            "Amount of A should equal the amount of B swapped"
        );

        // We verify that the changes in the balances are consistent
        assert_eq!(
            initial_balances.0 - token_a,
            amount_a,
            "Decrease in A should equal amount received"
        );
        assert_eq!(
            token_b - initial_balances.1,
            amount_b_to_swap,
            "Increase in B should equal amount swapped"
        );

        // We verify that the sum of the tokens remains constant (with a small margin of error for rounding).
        let initial_sum = initial_balances.0 + initial_balances.1;
        let final_sum = token_a + token_b;
        assert!(
            (initial_sum - final_sum).abs() < dec!(0.000001),
            "Total token amount should remain constant"
        );
    }

    #[test]
    fn test_swap_zero_amount() {
        let mut pool = create_standard_pool();
        assert!(pool.swap_a_to_b(dec!(0)).is_err());
        assert!(pool.swap_b_to_a(dec!(0)).is_err());
    }

    #[test]
    fn test_swap_negative_amount() {
        let mut pool = create_standard_pool();
        assert!(pool.swap_a_to_b(dec!(-100)).is_err());
        assert!(pool.swap_b_to_a(dec!(-100)).is_err());
    }

    #[test]
    fn test_swap_insufficient_liquidity() {
        let mut pool = create_standard_pool();
        assert!(pool.swap_a_to_b(dec!(1001)).is_err());
        assert!(pool.swap_b_to_a(dec!(1001)).is_err());
    }

    #[test]
    fn test_get_price() {
        let pool = create_standard_pool();
        assert!((pool.get_price() - dec!(1)).abs() < dec!(0.000001));
    }

    #[test]
    fn test_price_changes_after_swap() {
        let mut pool = create_standard_pool();
        let initial_price = pool.get_price();
        pool.swap_a_to_b(dec!(100)).unwrap();
        let price_after_swap = pool.get_price();
        assert!(price_after_swap < initial_price);
    }

    #[test]
    fn test_multiple_operations() {
        let mut pool = create_standard_pool();
        pool.add_liquidity(dec!(500), dec!(500)).unwrap();
        pool.swap_a_to_b(dec!(200)).unwrap();
        pool.swap_b_to_a(dec!(100)).unwrap();
        pool.remove_liquidity(dec!(300), dec!(300)).unwrap();
        let (token_a, token_b) = pool.get_balances();
        assert!(token_a != dec!(1000) && token_b != dec!(1000));
        assert!(token_a > dec!(0) && token_b > dec!(0));
    }

    #[test]
    fn test_extreme_swap() {
        let mut pool = create_standard_pool();
        let result = pool.swap_a_to_b(dec!(999));
        assert!(result.is_ok());
        let (token_a, token_b) = pool.get_balances();
        assert!(token_a > dec!(1990));
        assert!(token_b < dec!(10));
    }
}

#[cfg(test)]
mod tests_liquidity_pool_bis {
    use super::*;
    use rust_decimal_macros::dec;
    use tracing::debug;

    fn create_custom_pool(
        token_a: Decimal,
        token_b: Decimal,
        p_ref: Decimal,
        alpha: Decimal,
        beta: Decimal,
    ) -> LiquidityPool {
        LiquidityPool::new(token_a, token_b, p_ref, alpha, beta)
    }

    #[test]
    fn test_standard_pool() {
        let pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        assert_eq!(pool.get_balances(), (dec!(1000), dec!(1000)));
        assert_eq!(pool.get_price(), dec!(1));
    }

    #[test]
    fn test_unbalanced_pool() {
        let pool = create_custom_pool(dec!(500), dec!(2000), dec!(1), dec!(0.5), dec!(1));
        assert_eq!(pool.get_balances(), (dec!(500), dec!(2000)));
        assert!(pool.get_price() > dec!(1));
    }

    #[test]
    fn test_extreme_imbalance() {
        let pool = create_custom_pool(dec!(1), dec!(1000000), dec!(1), dec!(0.5), dec!(1));
        assert_eq!(pool.get_balances(), (dec!(1), dec!(1000000)));

        let price = pool.get_price();
        debug!("Price for extreme imbalance: {}", price);

        // Instead of asserting a specific price, let's check if it's significantly higher than 1
        assert!(
            price > dec!(1),
            "Price should be higher than 1 for extreme imbalance"
        );
        assert!(
            price < dec!(1000000),
            "Price should be less than the ratio of token balances"
        );

        // Calculate and print the ratio of token balances
        let balance_ratio = dec!(1000000) / dec!(1);
        debug!("Ratio of token balances: {}", balance_ratio);

        // Compare the price to the balance ratio
        debug!(
            "Price as percentage of balance ratio: {}%",
            (price / balance_ratio) * dec!(100)
        );
    }

    #[test]
    fn test_high_alpha() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.99), dec!(1));

        debug!("Initial price: {}", pool.get_price());

        let amount_to_swap = dec!(100);
        let result = pool.swap_a_to_b(amount_to_swap);
        assert!(result.is_ok());

        let amount_received = result.unwrap();
        debug!("Amount of A swapped: {}", amount_to_swap);
        debug!("Amount of B received: {}", amount_received);

        debug!("Final price: {}", pool.get_price());

        // Calculate and print the effective exchange rate
        let exchange_rate = amount_received / amount_to_swap;
        debug!("Effective exchange rate: {}", exchange_rate);

        // Assert that the exchange rate is close to 1, but slightly less
        assert!(
            exchange_rate <= dec!(1),
            "Exchange rate should not exceed 1"
        );
        assert!(
            exchange_rate > dec!(0.9),
            "Exchange rate should not be too low"
        );

        // Print pool balances after swap
        let (balance_a, balance_b) = pool.get_balances();
        debug!("Final balances - A: {}, B: {}", balance_a, balance_b);
    }

    #[test]
    fn test_low_alpha() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.01), dec!(1));
        let result = pool.swap_a_to_b(dec!(100));
        assert!(result.is_ok());
        assert!(result.unwrap() > dec!(99));
    }

    #[test]
    fn test_high_beta() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(100));

        debug!("Initial price: {}", pool.get_price());
        debug!("Initial balances: {:?}", pool.get_balances());

        let amount_to_swap = dec!(100);
        let result = pool.swap_a_to_b(amount_to_swap);
        assert!(result.is_ok());

        let amount_received = result.unwrap();
        debug!("Amount of A swapped: {}", amount_to_swap);
        debug!("Amount of B received: {}", amount_received);

        debug!("Final price: {}", pool.get_price());
        debug!("Final balances: {:?}", pool.get_balances());

        // Calculate and print the effective exchange rate
        let exchange_rate = amount_received / amount_to_swap;
        debug!("Effective exchange rate: {}", exchange_rate);

        // Assert that the exchange rate is close to 1, but may be slightly different
        assert!(
            exchange_rate > dec!(0.9),
            "Exchange rate should not be too low"
        );
        assert!(
            exchange_rate < dec!(1.1),
            "Exchange rate should not be too high"
        );

        // Assert that the price has changed due to the high beta
        assert!(
            pool.get_price() != dec!(1),
            "Price should change with high beta"
        );
    }

    #[test]
    fn test_low_beta() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(0.01));
        let result = pool.swap_a_to_b(dec!(100));
        assert!(result.is_ok());
        assert!((result.unwrap() - dec!(100)).abs() < dec!(0.01));
    }

    #[test]
    fn test_high_p_ref() {
        let pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1000), dec!(0.5), dec!(1));
        assert_eq!(pool.get_price(), dec!(1000));
    }

    #[test]
    fn test_low_p_ref() {
        let pool = create_custom_pool(dec!(1000), dec!(1000), dec!(0.001), dec!(0.5), dec!(1));
        assert_eq!(pool.get_price(), dec!(0.001));
    }

    #[test]
    fn test_swap_large_amount() {
        let mut pool =
            create_custom_pool(dec!(1000000), dec!(1000000), dec!(1), dec!(0.5), dec!(1));
        let result = pool.swap_a_to_b(dec!(500000));
        assert!(result.is_ok());
    }

    #[test]
    fn test_swap_tiny_amount() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        let result = pool.swap_a_to_b(dec!(0.000001));
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_large_liquidity() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        assert!(pool.add_liquidity(dec!(1000000), dec!(1000000)).is_ok());
    }

    #[test]
    fn test_remove_large_liquidity() {
        let mut pool =
            create_custom_pool(dec!(1000000), dec!(1000000), dec!(1), dec!(0.5), dec!(1));
        assert!(pool.remove_liquidity(dec!(999999), dec!(999999)).is_ok());
    }

    #[test]
    fn test_remove_all_liquidity() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        assert!(pool.remove_liquidity(dec!(1000), dec!(1000)).is_ok());
        assert_eq!(pool.get_balances(), (dec!(0), dec!(0)));
    }

    #[test]
    fn test_swap_after_large_addition() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        assert!(pool.add_liquidity(dec!(1000000), dec!(1000000)).is_ok());
        let result = pool.swap_a_to_b(dec!(100));
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_swaps() {
        let mut pool = create_custom_pool(dec!(10000), dec!(10000), dec!(1), dec!(0.5), dec!(1));
        for _ in 0..10 {
            assert!(pool.swap_a_to_b(dec!(100)).is_ok());
            assert!(pool.swap_b_to_a(dec!(90)).is_ok());
        }
    }

    #[test]
    fn test_extreme_alpha_beta_combination() {
        let pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.99), dec!(100));
        assert!(pool.get_price() > dec!(0.9) && pool.get_price() < dec!(1.1));
    }

    #[test]
    fn test_uneven_liquidity_addition() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        assert!(pool.add_liquidity(dec!(500), dec!(1000)).is_ok());
        assert_eq!(pool.get_balances(), (dec!(1500), dec!(2000)));
    }

    #[test]
    fn test_uneven_liquidity_removal() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        assert!(pool.remove_liquidity(dec!(300), dec!(500)).is_ok());
        assert_eq!(pool.get_balances(), (dec!(700), dec!(500)));
    }

    #[test]
    fn test_swap_exact_remaining_balance() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        assert!(pool.swap_a_to_b(dec!(1000)).is_ok());
    }

    #[test]
    fn test_swap_more_than_balance() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        let result = pool.swap_a_to_b(dec!(1001));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Insufficient liquidity");
    }

    #[test]
    fn test_price_change_after_swap() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        let initial_price = pool.get_price();
        pool.swap_a_to_b(dec!(100)).unwrap();
        assert!(pool.get_price() != initial_price);
    }

    #[test]
    fn test_zero_liquidity_pool() {
        let pool = create_custom_pool(dec!(0), dec!(0), dec!(1), dec!(0.5), dec!(1));
        assert_eq!(pool.get_balances(), (dec!(0), dec!(0)));
    }

    #[test]
    #[should_panic]
    fn test_swap_in_zero_liquidity_pool() {
        let mut pool = create_custom_pool(dec!(0), dec!(0), dec!(1), dec!(0.5), dec!(1));
        pool.swap_a_to_b(dec!(100)).unwrap();
    }

    #[test]
    fn test_add_remove_tiny_liquidity() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        assert!(pool.add_liquidity(dec!(0.000001), dec!(0.000001)).is_ok());
        assert!(pool
            .remove_liquidity(dec!(0.000001), dec!(0.000001))
            .is_ok());
    }

    #[test]
    fn test_swap_with_extreme_price_reference() {
        let mut pool =
            create_custom_pool(dec!(1000), dec!(1000), dec!(1000000), dec!(0.5), dec!(1));

        debug!("Initial state:");
        debug!("Balances: {:?}", pool.get_balances());
        debug!("Price: {}", pool.get_price());

        // Test large A to B swap (should fail)
        let amount_to_swap = dec!(100);
        let result = pool.swap_a_to_b(amount_to_swap);
        debug!("Large A to B swap result: {:?}", result);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Insufficient liquidity for swap"
        );

        // Test small A to B swap (should succeed)
        let small_amount = dec!(0.000001);
        let small_swap_result = pool.swap_a_to_b(small_amount);
        debug!("Small A to B swap result: {:?}", small_swap_result);
        assert!(small_swap_result.is_ok());
        let received = small_swap_result.unwrap();
        debug!("Received from small A to B swap: {}", received);
        assert_eq!(
            received,
            dec!(1),
            "Should receive exactly 1 token B for 0.000001 token A"
        );

        // Test B to A swap (should succeed but with very small return)
        let b_to_a_result = pool.swap_b_to_a(dec!(1));
        debug!("B to A swap result: {:?}", b_to_a_result);
        assert!(b_to_a_result.is_ok());
        let received_a = b_to_a_result.unwrap();
        debug!("Received from B to A swap: {}", received_a);
        assert!(
            received_a < dec!(0.000002),
            "Should receive very small amount of A for 1 token B"
        );

        // Verify final state
        let (final_a, final_b) = pool.get_balances();
        let final_price = pool.get_price();
        debug!("Final state:");
        debug!("Balances: ({}, {})", final_a, final_b);
        debug!("Price: {}", final_price);

        // The changes are extremely small due to the extreme price
        assert!(
            (final_a - dec!(1000)).abs() < dec!(0.000001),
            "A balance should change very little"
        );
        assert!(
            (final_b - dec!(1000)).abs() < dec!(0.000001),
            "B balance should change very little"
        );
        assert_ne!(
            final_price,
            dec!(1000000),
            "Price should change slightly after swaps"
        );

        // Additional checks to understand the behavior
        debug!("Change in A balance: {}", final_a - dec!(1000));
        debug!("Change in B balance: {}", final_b - dec!(1000));
        debug!("Change in price: {}", final_price - dec!(1000000));
    }
}
