/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use crate::arpp::liquidity_pool::LiquidityPool;
use rand::Rng;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
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
pub struct RandomStrategy {
    swap_probability: f64,
    max_swap_amount: Decimal,
}

impl RandomStrategy {
    pub fn new(swap_probability: f64, max_swap_amount: Decimal) -> Self {
        Self {
            swap_probability,
            max_swap_amount,
        }
    }
}

impl TradingStrategy for RandomStrategy {
    /// Executes a stochastic swap operation within a given liquidity pool.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the struct or instance which implements this function.
    /// * `pool` - A mutable reference to a `LiquidityPool` where the operation will take place.
    /// * `_` - A `Decimal` value, not currently used in this function but reserved for future use.
    ///
    /// # Returns
    ///
    /// A `Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>>` representing the async result
    /// of the operation. The operation may succeed or yield an error encapsulated in a `Box<dyn Error>`.
    ///
    /// # Details
    ///
    /// This function uses a randomly generated number to determine if a swap should occur based on a
    /// predefined swap probability (`self.swap_probability`). If the swap condition is met, it
    /// retrieves the balances of two assets within the pool and randomly decides to swap a certain
    /// amount from one asset to the other. The swap amount is bounded by half of each balance or a
    /// predefined maximum swap amount (`self.max_swap_amount`).
    ///
    /// During the operation, debug logs are generated to indicate the direction of the swap and the
    /// amount being swapped.
    ///
    fn execute<'a>(
        &'a self,
        pool: &'a mut LiquidityPool,
        _: Decimal,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>> {
        Box::pin(async move {
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < self.swap_probability {
                let (balance_a, balance_b) = pool.get_balances();

                let amount_a = (balance_a / Decimal::new(2, 0)).min(self.max_swap_amount); // 50% del balance de A
                let amount_b = (balance_b / Decimal::new(2, 0)).min(self.max_swap_amount); // 50% del balance de B

                let amount = Decimal::from_f64(rng.gen::<f64>()).unwrap() * self.max_swap_amount;

                if rng.gen::<bool>() {
                    let swap_amount = amount.min(amount_a);
                    debug!("Swapping {} tokens from A to B", swap_amount);
                    pool.swap_a_to_b(swap_amount)?;
                } else {
                    let swap_amount = amount.min(amount_b);
                    debug!("Swapping {} tokens from B to A", swap_amount);
                    pool.swap_b_to_a(swap_amount)?;
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
            if current_price > pool.get_p_ref() + self.swap_threshold {
                pool.swap_b_to_a(self.swap_amount)?;
            } else if current_price < pool.get_p_ref() - self.swap_threshold {
                pool.swap_a_to_b(self.swap_amount)?;
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
        let strategy = RandomStrategy::new(0.5, dec!(100));
        assert_eq!(strategy.swap_probability, 0.5);
        assert_eq!(strategy.max_swap_amount, dec!(100));
    }

    #[tokio::test]
    async fn test_random_strategy_execution() {
        let strategy = RandomStrategy::new(1.0, dec!(50)); // Always swap
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
    async fn test_random_strategy_no_execution() {
        let strategy = RandomStrategy::new(0.0, dec!(50)); // Never swap
        let pool = create_mock_pool();
        let mut pool_guard = pool.lock().await;
        let initial_balance = pool_guard.get_balances();

        strategy.execute(&mut pool_guard, dec!(1)).await.unwrap();

        let final_balance = pool_guard.get_balances();
        assert_eq!(initial_balance, final_balance, "Balances should not change");
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
