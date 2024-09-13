/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use crate::arpp::liquidity_pool::LiquidityPool;
use crate::utils::helpers::random_decimal;
use rand::prelude::SliceRandom;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use tracing::debug;

/// A trait for defining trading strategies in a liquidity pool context.
///
/// This trait requires the implementation of the `execute` method, which will
/// contain the logic for the trading strategy. Implementors of this trait must
/// be thread-safe (i.e., implement `Send` and `Sync`).
///
/// # Methods
///
/// - `execute`: Executes the trading strategy with the given liquidity pool and
///   current price. The method returns a `Future` that will produce a `Result`.
///
/// # Arguments
///
/// * `pool` - A mutable reference to a `LiquidityPool`, representing the pool
///   of liquidity where trades are conducted.
/// * `current_price` - A `Decimal` representing the current price of the asset.
///
/// # Returns
///
/// A `Future` that resolves to a `Result<(), Box<dyn Error>>`, indicating the
/// success or failure of the strategy execution.
#[allow(clippy::type_complexity)]
pub trait TradingStrategy: Send + Sync {
    fn execute<'a>(
        &'a self,
        pool: &'a mut LiquidityPool,
        current_price: Decimal,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>>;
}

/// A struct representing a strategy that uses randomness for decision making.
///
/// The `RandomStrategy` struct contains parameters that define the behavior of the strategy,
/// such as the probability of making a swap and the maximum amount to swap.
///
/// # Fields
///
/// * `swap_probability` - A `f64` that represents the probability of making a swap.
/// * `max_swap_amount` - A `Decimal` that specifies the maximum amount that can be swapped.
#[allow(dead_code)]
pub struct RandomStrategy {
    balance_a: Decimal,
    balance_b: Decimal,
}

impl RandomStrategy {
    pub fn new(balance_a: Decimal, balance_b: Decimal) -> Self {
        Self {
            balance_a,
            balance_b,
        }
    }
}

impl TradingStrategy for RandomStrategy {
    fn execute<'a>(
        &'a self,
        pool: &'a mut LiquidityPool,
        _: Decimal,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>> {
        Box::pin(async move {
            let mut rng = rand::thread_rng();
            let list = [1, 2, 3];
            let random_number = list.choose(&mut rng).expect("Shouldn't be empty");
            let (balance_a, balance_b) = pool.get_balances();
            let splitter = dec!(1000);

            match random_number {
                1 => {
                    let swap_amount = random_decimal(balance_a) / splitter;
                    debug!("Swapping {:.4} tokens from A to B", swap_amount);
                    pool.swap_a_to_b(swap_amount)?;
                }
                2 => {
                    let swap_amount = random_decimal(balance_b) / splitter;
                    debug!("Swapping {:.4} tokens from B to A", swap_amount);
                    pool.swap_b_to_a(swap_amount)?;
                }
                3 => {
                    let (balance_a, balance_b) = pool.get_balances();

                    let sum = balance_a + balance_b;
                    let ratio = balance_a / balance_b;

                    let diff = (self.balance_a + self.balance_b - sum).abs();

                    if ratio > dec!(1.05) {
                        let swap_amount = self.balance_b * dec!(0.1);
                        debug!("Adding liquidity to pool Token A: {:.4}", swap_amount);
                        pool.add_liquidity(dec!(0), swap_amount)?;
                    }
                    if ratio < dec!(0.95) {
                        let swap_amount = self.balance_a * dec!(0.1);
                        debug!("Adding liquidity to pool Token B: {:.4}", swap_amount);
                        pool.add_liquidity(swap_amount, dec!(0))?;
                    }

                    if sum < (self.balance_a + self.balance_b) {
                        debug!("Adding liquidity to pool: {:.4}", diff / dec!(2));
                        pool.add_liquidity(diff / dec!(2), diff / dec!(2))?;
                    }

                    if sum > (self.balance_a + self.balance_b) {
                        debug!("Removing liquidity from pool: {:.4}", diff / dec!(2));
                        pool.remove_liquidity(diff / dec!(2), diff / dec!(2))?;
                    }
                }
                _ => {
                    debug!("No swap");
                }
            }
            Ok(())
        })
    }
}

/// A strategy for mean reversion trading.
///
/// `MeanReversionStrategy` is used to manage swaps based on the mean reversion principle,
/// which counters trends and takes advantage of price oscillations.
///
/// # Fields
///
/// * `swap_threshold` - The threshold value at which a swap should be triggered.
/// * `swap_amount` - The amount to be swapped when the swap threshold is reached.
///
pub struct MeanReversionStrategy {
    swap_threshold: Decimal,
    swap_amount: Decimal,
}

impl MeanReversionStrategy {
    pub fn new(swap_threshold: Decimal, swap_amount: Decimal) -> Self {
        Self {
            swap_threshold,
            swap_amount,
        }
    }
}

impl TradingStrategy for MeanReversionStrategy {
    /// Executes a liquidity pool swap operation asynchronously based on the current price.
    ///
    /// This function will initiate a swap from token B to token A or vice versa, depending
    /// on the current price relative to a reference price (`p_ref`) plus or minus a threshold.
    ///
    /// # Arguments
    ///
    /// * `pool` - A mutable reference to the `LiquidityPool` instance where the swap
    ///            operations will occur.
    /// * `current_price` - A `Decimal` representing the current price of the token.
    ///
    /// # Returns
    ///
    /// A pinned `Box` containing a `Future` which resolves to a `Result` type:
    /// * `Ok(())` - If the swap operation is executed successfully.
    /// * `Err(Box<dyn Error>)` - If an error occurs during the swap operation.
    ///
    fn execute<'a>(
        &'a self,
        pool: &'a mut LiquidityPool,
        current_price: Decimal,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>> {
        Box::pin(async move {
            let diff = current_price / pool.get_p_ref();
            if current_price > pool.get_p_ref() * (dec!(1) + self.swap_threshold) {
                pool.swap_b_to_a(diff * self.swap_amount)?;
            } else if current_price < pool.get_p_ref() * (dec!(1) - self.swap_threshold) {
                pool.swap_a_to_b(diff * self.swap_amount)?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests_trading_strategy {
    use super::*;
    use rust_decimal_macros::dec;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Helper function to create a mock LiquidityPool
    fn create_mock_pool() -> Arc<Mutex<LiquidityPool>> {
        Arc::new(Mutex::new(LiquidityPool::new(
            dec!(1000),
            dec!(1000),
            dec!(1),
            dec!(0.5),
            dec!(1),
        )))
    }

    #[tokio::test]
    async fn test_random_strategy_creation() {
        let strategy = RandomStrategy::new(dec!(100), dec!(100));
        assert_eq!(strategy.balance_a, dec!(100));
        assert_eq!(strategy.balance_b, dec!(100));
    }

    #[tokio::test]
    async fn test_random_strategy_execution() {
        let strategy = RandomStrategy::new(dec!(100), dec!(50)); // Always swap
        let pool = create_mock_pool();
        let mut pool_guard = pool.lock().await;
        let initial_balance = pool_guard.get_balances();

        strategy.execute(&mut pool_guard, dec!(1)).await.unwrap();

        let final_balance = pool_guard.get_balances();
        assert_ne!(
            initial_balance, final_balance,
            "Balances should change after swap"
        );
    }

    #[tokio::test]
    async fn test_mean_reversion_strategy_creation() {
        let strategy = MeanReversionStrategy::new(dec!(0.1), dec!(10));
        assert_eq!(strategy.swap_threshold, dec!(0.1));
        assert_eq!(strategy.swap_amount, dec!(10));
    }

    #[tokio::test]
    async fn test_mean_reversion_strategy_above_threshold() {
        let strategy = MeanReversionStrategy::new(dec!(0.1), dec!(10));
        let pool = create_mock_pool();
        let mut pool_guard = pool.lock().await;
        let initial_balance = pool_guard.get_balances();

        strategy.execute(&mut pool_guard, dec!(1.2)).await.unwrap();

        let final_balance = pool_guard.get_balances();
        assert_ne!(
            initial_balance, final_balance,
            "Balances should change after swap"
        );
        assert!(
            final_balance.0 < initial_balance.0,
            "Token A balance should decrease"
        );
        assert!(
            final_balance.1 > initial_balance.1,
            "Token B balance should increase"
        );
    }

    #[tokio::test]
    async fn test_mean_reversion_strategy_below_threshold() {
        let strategy = MeanReversionStrategy::new(dec!(0.1), dec!(10));
        let pool = create_mock_pool();
        let mut pool_guard = pool.lock().await;
        let initial_balance = pool_guard.get_balances();

        strategy.execute(&mut pool_guard, dec!(0.8)).await.unwrap();

        let final_balance = pool_guard.get_balances();
        assert_ne!(
            initial_balance, final_balance,
            "Balances should change after swap"
        );
        assert!(
            final_balance.0 > initial_balance.0,
            "Token A balance should increase"
        );
        assert!(
            final_balance.1 < initial_balance.1,
            "Token B balance should decrease"
        );
    }

    #[tokio::test]
    async fn test_mean_reversion_strategy_within_threshold() {
        let strategy = MeanReversionStrategy::new(dec!(0.1), dec!(10));
        let pool = create_mock_pool();
        let mut pool_guard = pool.lock().await;
        let initial_balance = pool_guard.get_balances();

        strategy.execute(&mut pool_guard, dec!(1.05)).await.unwrap();

        let final_balance = pool_guard.get_balances();
        assert_eq!(initial_balance, final_balance, "Balances should not change");
    }
}
