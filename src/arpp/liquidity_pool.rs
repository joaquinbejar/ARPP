/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 10/9/24
 ******************************************************************************/

use crate::arpp::formula::arpp;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;

#[derive(Debug)]
pub struct LiquidityPool {
    token_a: Decimal,
    token_b: Decimal,
    p_ref: Decimal,
    alpha: Decimal,
    beta: Decimal,
}

impl LiquidityPool {
    pub fn new(token_a: Decimal, token_b: Decimal, p_ref: Decimal, alpha: Decimal, beta: Decimal) -> Self {
        Self {
            token_a,
            token_b,
            p_ref,
            alpha,
            beta,
        }
    }

    pub fn add_liquidity(&mut self, amount_a: Decimal, amount_b: Decimal) -> Result<(), Box<dyn Error>> {
        if amount_a <= Decimal::ZERO || amount_b <= Decimal::ZERO {
            return Err("Amounts must be positive".into());
        }
        self.token_a += amount_a;
        self.token_b += amount_b;
        Ok(())
    }

    pub fn remove_liquidity(&mut self, amount_a: Decimal, amount_b: Decimal) -> Result<(), Box<dyn Error>> {
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

    pub fn get_price(&self) -> Decimal {
        let r = self.token_b / self.token_a;
        arpp(self.p_ref, self.alpha, self.beta, r)
    }

    pub fn get_balances(&self) -> (Decimal, Decimal) {
        (self.token_a, self.token_b)
    }
}

#[cfg(test)]
mod tests_liquidity_pool {
    use super::*;

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
        println!("Initial balances: {:?}", initial_balances);

        let amount_a_to_swap = dec!(100);
        let result = pool.swap_a_to_b(amount_a_to_swap);
        assert!(result.is_ok());
        let amount_b = result.unwrap();

        let (token_a, token_b) = pool.get_balances();
        println!("Final balances: ({}, {})", token_a, token_b);
        println!("Amount of B received: {}", amount_b);

        assert!(token_a > initial_balances.0, "Token A should increase");
        assert!(token_b < initial_balances.1, "Token B should decrease");

        // Verify that the amount of B received equals the amount of A swapped
        assert_eq!(amount_b, amount_a_to_swap, "Amount of B should equal the amount of A swapped");

        // Check that the changes in balances are consistent
        assert_eq!(token_a - initial_balances.0, amount_a_to_swap, "Increase in A should equal amount swapped");
        assert_eq!(initial_balances.1 - token_b, amount_b, "Decrease in B should equal amount received");

        // Verify that the sum of tokens remains constant (with a small margin for rounding errors)
        let initial_sum = initial_balances.0 + initial_balances.1;
        let final_sum = token_a + token_b;
        assert!((initial_sum - final_sum).abs() < dec!(0.000001), "Total token amount should remain constant");

        let swap_rate = amount_b / amount_a_to_swap;
        println!("Swap rate (B/A): {}", swap_rate);
        assert_eq!(swap_rate, dec!(1), "Swap rate should be 1:1");
    }

    #[test]
    fn test_swap_b_to_a() {
        let mut pool = create_standard_pool();
        let initial_balances = pool.get_balances();
        println!("Initial balances: {:?}", initial_balances);

        let amount_b_to_swap = dec!(100);
        let result = pool.swap_b_to_a(amount_b_to_swap);
        assert!(result.is_ok());
        let amount_a = result.unwrap();

        let (token_a, token_b) = pool.get_balances();
        println!("Final balances: ({}, {})", token_a, token_b);
        println!("Amount of A received: {}", amount_a);

        assert!(token_a < initial_balances.0, "Token A should decrease");
        assert!(token_b > initial_balances.1, "Token B should increase");

        // We verify that the amount of A received is equal to the amount of B delivered.
        assert_eq!(amount_a, amount_b_to_swap, "Amount of A should equal the amount of B swapped");

        // We verify that the changes in the balances are consistent
        assert_eq!(initial_balances.0 - token_a, amount_a, "Decrease in A should equal amount received");
        assert_eq!(token_b - initial_balances.1, amount_b_to_swap, "Increase in B should equal amount swapped");

        // We verify that the sum of the tokens remains constant (with a small margin of error for rounding).
        let initial_sum = initial_balances.0 + initial_balances.1;
        let final_sum = token_a + token_b;
        assert!((initial_sum - final_sum).abs() < dec!(0.000001), "Total token amount should remain constant");
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

    fn create_custom_pool(token_a: Decimal, token_b: Decimal, p_ref: Decimal, alpha: Decimal, beta: Decimal) -> LiquidityPool {
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
        assert!(pool.get_price() > dec!(1000));
    }

    #[test]
    fn test_high_alpha() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.99), dec!(1));
        let result = pool.swap_a_to_b(dec!(100));
        assert!(result.is_ok());
        assert!(result.unwrap() < dec!(100));
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
        let result = pool.swap_a_to_b(dec!(100));
        assert!(result.is_ok());
        assert!(result.unwrap() < dec!(100));
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
        let mut pool = create_custom_pool(dec!(1000000), dec!(1000000), dec!(1), dec!(0.5), dec!(1));
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
        let mut pool = create_custom_pool(dec!(1000000), dec!(1000000), dec!(1), dec!(0.5), dec!(1));
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
    #[should_panic]
    fn test_swap_more_than_balance() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1), dec!(0.5), dec!(1));
        pool.swap_a_to_b(dec!(1001)).unwrap();
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
        assert!(pool.remove_liquidity(dec!(0.000001), dec!(0.000001)).is_ok());
    }

    #[test]
    fn test_swap_with_extreme_price_reference() {
        let mut pool = create_custom_pool(dec!(1000), dec!(1000), dec!(1000000), dec!(0.5), dec!(1));
        let result = pool.swap_a_to_b(dec!(100));
        assert!(result.is_ok());
        assert!(result.unwrap() > dec!(100));
    }
}