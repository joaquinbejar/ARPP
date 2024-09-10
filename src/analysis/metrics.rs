/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/
use std::ops::Neg;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use crate::arpp::liquidity_pool::LiquidityPool;
use crate::simulation::monte_carlo::SimulationResult;


#[derive(Clone)]
pub struct PoolMetrics {
    pub price_volatility: Decimal,
    pub liquidity_depth: Decimal,
    pub trading_volume: Decimal,
    pub impermanent_loss: Decimal,
}


/// Calculates various metrics for a given liquidity pool compared to its initial state.
///
/// # Arguments
///
/// * `pool` - A reference to the current state of the `LiquidityPool`.
/// * `initial_pool` - A reference to the initial state of the `LiquidityPool`.
///
/// # Returns
///
/// * `PoolMetrics` - A struct containing the calculated metrics:
///     - `price_volatility`: The volatility of the pool's price.
///     - `liquidity_depth`: The depth of liquidity in the pool.
///     - `trading_volume`: The trading volume of the pool.
///     - `impermanent_loss`: The impermanent loss of the pool.
///
pub fn calculate_pool_metrics(pool: &LiquidityPool, initial_pool: &LiquidityPool) -> PoolMetrics {
    let (token_a, token_b) = pool.get_balances();
    let (initial_a, initial_b) = initial_pool.get_balances();
    let current_price = pool.get_price();
    let initial_price = initial_pool.get_price();

    PoolMetrics {
        price_volatility: calculate_price_volatility(current_price, initial_price),
        liquidity_depth: calculate_liquidity_depth(token_a, token_b),
        trading_volume: calculate_trading_volume(token_a, token_b, initial_a, initial_b),
        impermanent_loss: calculate_impermanent_loss(token_a, token_b, initial_a, initial_b, current_price, initial_price),
    }
}

/// Calculates the price volatility given the current price and initial price.
///
/// # Arguments
///
/// * `current_price` - A `Decimal` representing the current price of the asset.
/// * `initial_price` - A `Decimal` representing the initial price of the asset.
///
/// # Returns
///
/// A `Decimal` representing the absolute price volatility as a fraction.
pub fn analyze_simulation_results(results: &SimulationResult) -> SimulationAnalysis {
    SimulationAnalysis {
        price_stability: calculate_price_stability(results.min_price, results.max_price),
        average_price_impact: results.average_price_change,
        liquidity_efficiency: calculate_liquidity_efficiency(results.average_liquidity_change),
    }
}

/// Calculates the volatility of a price based on its current and initial values.
///
/// # Parameters
///
/// - `current_price`: The current price as a `Decimal`.
/// - `initial_price`: The initial price as a `Decimal`.
///
/// # Returns
///
/// A `Decimal` representing the price volatility, clamped between 0 and 1 (100%).
///
/// * Returns `1.0` if any of the input prices is negative, indicating invalid input.
/// * Returns `1.0` if the initial price is zero but the current price is non-zero, indicating maximum volatility.
/// * Returns `0.0` if both prices are zero, indicating no volatility.
/// * Otherwise, returns the absolute value of the relative change in price, clamped between 0 and 1.
///
fn calculate_price_volatility(current_price: Decimal, initial_price: Decimal) -> Decimal {
    // Check for non-negative inputs
    if current_price < Decimal::ZERO || initial_price < Decimal::ZERO {
        return Decimal::ONE; // Return maximum volatility for invalid inputs
    }

    // Handle division by zero for initial price
    if initial_price == Decimal::ZERO {
        if current_price == Decimal::ZERO {
            return Decimal::ZERO; // No volatility if both prices are zero
        } else {
            return Decimal::ONE; // Maximum volatility if initial price was zero but current is not
        }
    }

    // Calculate volatility
    let volatility = ((current_price - initial_price) / initial_price).abs();

    // Clamp the result to a reasonable range, e.g., 0 to 1
    // This assumes volatility should not exceed 100%
    volatility.clamp(Decimal::ZERO, Decimal::ONE)
}

/// Calculates the liquidity depth for two given tokens.
///
/// # Arguments
///
/// * `token_a` - The amount of the first token in the liquidity pool.
/// * `token_b` - The amount of the second token in the liquidity pool.
///
/// # Returns
///
/// A `Decimal` representing the liquidity depth, which is the square root of the product of `token_a` and `token_b`.
/// If the calculation fails, it returns a `Decimal` value of 0.
///
fn calculate_liquidity_depth(token_a: Decimal, token_b: Decimal) -> Decimal {
    let results = (token_a * token_b).sqrt();
    results.unwrap_or_else(|| dec!(0))
}

/// Calculates the trading volume of two tokens.
///
/// This function computes the absolute differences between the current quantities
/// (`token_a` and `token_b`) and their respective initial quantities (`initial_a` and `initial_b`).
/// Then it sums these differences to obtain the total trading volume.
///
/// # Arguments
///
/// * `token_a` - The current amount of token A.
/// * `token_b` - The current amount of token B.
/// * `initial_a` - The initial amount of token A.
/// * `initial_b` - The initial amount of token B.
///
/// # Returns
///
/// A `Decimal` value representing the total trading volume.
fn calculate_trading_volume(token_a: Decimal, token_b: Decimal, initial_a: Decimal, initial_b: Decimal) -> Decimal {
    (token_a - initial_a).abs() + (token_b - initial_b).abs()
}

/// Calculates the price stability based on the minimum and maximum prices provided.
///
/// This function evaluates how stable the prices are within a given range.
/// Stability is determined by comparing the variation between the minimum and maximum prices
/// to their average price. The result is a decimal value between 0 and 1, where 1 indicates perfect stability,
/// and 0 indicates extreme instability.
///
/// # Arguments
///
/// * `min_price` - A `Decimal` representing the minimum price in the range.
/// * `max_price` - A `Decimal` representing the maximum price in the range.
///
/// # Returns
///
/// A `Decimal` value between 0 and 1 representing the price stability.
/// A return value of 1 means perfect stability (no price movement), and 0 means extreme instability
/// or invalid price range.
///
/// # Notes
///
/// - If both `min_price` and `max_price` are zero, the function returns `Decimal::ONE` indicating perfect stability.
/// - If the price range is invalid (negative prices or `min_price` greater than `max_price`), the function returns `Decimal::ZERO`.
/// - The result is clamped to ensure it falls within the 0 to 1 range.
fn calculate_impermanent_loss(
    token_a: Decimal,
    token_b: Decimal,
    initial_a: Decimal,
    initial_b: Decimal,
    current_price: Decimal,
    initial_price: Decimal
) -> Decimal {
    // Check for non-negative inputs
    if token_a < Decimal::ZERO || token_b < Decimal::ZERO ||
        initial_a < Decimal::ZERO || initial_b < Decimal::ZERO ||
        current_price < Decimal::ZERO || initial_price < Decimal::ZERO {
        return Decimal::ZERO; // Return 0 for invalid inputs
    }

    // Handle division by zero for initial price
    if initial_price == Decimal::ZERO {
        if current_price == Decimal::ZERO {
            return Decimal::ZERO; // No change if both prices are zero
        } else {
            return Decimal::ONE; // Assume 100% loss if initial price was zero but current is not
        }
    }

    let value_if_held = initial_a * current_price / initial_price + initial_b;
    let value_in_pool = token_a * current_price + token_b;

    // Handle division by zero for value_if_held
    if value_if_held == Decimal::ZERO {
        if value_in_pool == Decimal::ZERO {
            return Decimal::ZERO; // No impermanent loss if both values are zero
        } else {
            return Decimal::ONE; // Assume 100% gain if held value is zero but pool value is not
        }
    }

    let impermanent_loss = (value_in_pool - value_if_held) / value_if_held;

    // Clamp the result to a reasonable range, e.g., -1 to 1
    // This assumes impermanent loss/gain should not exceed 100%
    impermanent_loss.clamp(Decimal::ONE.neg(), Decimal::ONE)
}

/// Calculates the price stability based on the minimum and maximum prices provided.
///
/// This function evaluates how stable the prices are within a given range.
/// Stability is determined by comparing the variation between the minimum and maximum prices
/// to their average price. The result is a decimal value between 0 and 1, where 1 indicates perfect stability,
/// and 0 indicates extreme instability.
///
/// # Arguments
///
/// * `min_price` - A `Decimal` representing the minimum price in the range.
/// * `max_price` - A `Decimal` representing the maximum price in the range.
///
/// # Returns
///
/// A `Decimal` value between 0 and 1 representing the price stability.
/// A return value of 1 means perfect stability (no price movement), and 0 means extreme instability
/// or invalid price range.
/// # Notes
///
/// - If both `min_price` and `max_price` are zero, the function returns `Decimal::ONE` indicating perfect stability.
/// - If the price range is invalid (negative prices or `min_price` greater than `max_price`), the function returns `Decimal::ZERO`.
/// - The result is clamped to ensure it falls within the 0 to 1 range.
fn calculate_price_stability(min_price: Decimal, max_price: Decimal) -> Decimal {
    // Handle the case where both prices are zero
    if min_price == Decimal::ZERO && max_price == Decimal::ZERO {
        return Decimal::ONE; // Perfect stability when there's no price movement
    }

    // Handle the case where prices are negative or min is greater than max
    if min_price < Decimal::ZERO || max_price < Decimal::ZERO || min_price > max_price {
        return Decimal::ZERO; // Invalid price range, return minimum stability
    }

    let avg_price = (max_price + min_price) / Decimal::TWO;

    // Avoid division by zero if avg_price is zero
    if avg_price == Decimal::ZERO {
        return Decimal::ZERO; // Extreme instability when average price is zero
    }

    let stability = Decimal::ONE - (max_price - min_price) / avg_price;

    // Ensure the result is between 0 and 1
    stability.clamp(Decimal::ZERO, Decimal::ONE)
}

/// Calculates the liquidity efficiency based on the average liquidity change.
///
/// This function evaluates the liquidity efficiency using the given average
/// liquidity change. If the average liquidity change is -1, the efficiency
/// is returned as 0. Otherwise, the efficiency is computed as the reciprocal
/// of (1 + average liquidity change) and clamped between 0 and 1.
///
/// # Arguments
///
/// * `average_liquidity_change` - A `Decimal` value representing the average
///   change in liquidity.
///
/// # Returns
///
/// * A `Decimal` value representing the efficiency, clamped between 0 and 1.
fn calculate_liquidity_efficiency(average_liquidity_change: Decimal) -> Decimal {
    if average_liquidity_change == Decimal::ONE.neg() {
        return Decimal::ZERO;
    }
    let efficiency = Decimal::ONE / (Decimal::ONE + average_liquidity_change);
    efficiency.clamp(Decimal::ZERO, Decimal::ONE)
}

pub struct SimulationAnalysis {
    pub price_stability: Decimal,
    pub average_price_impact: Decimal,
    pub liquidity_efficiency: Decimal,
}



#[cfg(test)]
mod tests_price_volatility {
    use super::*;
    use rust_decimal_macros::dec;
    use tracing::info;
    use crate::utils::logger::setup_logger;

    fn test_price_volatility(current_price: Decimal, initial_price: Decimal) {
        let volatility = calculate_price_volatility(current_price, initial_price);
        info!(
            "Current Price: {}, Initial Price: {}, Volatility: {}",
            current_price, initial_price, volatility
        );
    }

    #[test]
    fn test_volatility() {
        setup_logger();
        // No change
        test_price_volatility(dec!(100), dec!(100));

        // 50% increase
        test_price_volatility(dec!(150), dec!(100));

        // 50% decrease
        test_price_volatility(dec!(50), dec!(100));

        // 100% increase
        test_price_volatility(dec!(200), dec!(100));

        // Large increase (beyond 100%)
        test_price_volatility(dec!(300), dec!(100));

        // Zero initial price
        test_price_volatility(dec!(100), dec!(0));

        // Zero current price
        test_price_volatility(dec!(0), dec!(100));

        // Both prices zero
        test_price_volatility(dec!(0), dec!(0));

        // Negative current price
        test_price_volatility(dec!(-100), dec!(100));

        // Negative initial price
        test_price_volatility(dec!(100), dec!(-100));

        // Very small price change
        test_price_volatility(dec!(1.0001), dec!(1));

        // Very large numbers
        test_price_volatility(dec!(1000000), dec!(1000));
    }
}

#[cfg(test)]
mod tests_calculate_impermanent_loss {
    use super::*;
    use rust_decimal_macros::dec;
    use tracing::info;
    use crate::utils::logger::setup_logger;

    fn test_impermanent_loss(
        token_a: Decimal,
        token_b: Decimal,
        initial_a: Decimal,
        initial_b: Decimal,
        current_price: Decimal,
        initial_price: Decimal
    ) {
        let loss = calculate_impermanent_loss(token_a, token_b, initial_a, initial_b, current_price, initial_price);
        info!(
            "Token A: {}, Token B: {}, Initial A: {}, Initial B: {}, Current Price: {}, Initial Price: {}, Impermanent Loss: {}",
            token_a, token_b, initial_a, initial_b, current_price, initial_price, loss
        );
    }

    #[test]
    fn tests_calculate_impermanent_loss() {
        setup_logger();

        // Normal case
        test_impermanent_loss(dec!(90), dec!(110), dec!(100), dec!(100), dec!(1.1), dec!(1.0));

        // No price change
        test_impermanent_loss(dec!(100), dec!(100), dec!(100), dec!(100), dec!(1.0), dec!(1.0));

        // Extreme price change
        test_impermanent_loss(dec!(50), dec!(200), dec!(100), dec!(100), dec!(2.0), dec!(1.0));

        // Zero initial price
        test_impermanent_loss(dec!(100), dec!(100), dec!(100), dec!(100), dec!(1.0), dec!(0));

        // Zero current price
        test_impermanent_loss(dec!(100), dec!(100), dec!(100), dec!(100), dec!(0), dec!(1.0));

        // Negative input
        test_impermanent_loss(dec!(-100), dec!(100), dec!(100), dec!(100), dec!(1.0), dec!(1.0));

        // Very large numbers
        test_impermanent_loss(dec!(1000000), dec!(1000000), dec!(1000000), dec!(1000000), dec!(2.0), dec!(1.0));
    }
}

#[cfg(test)]
mod tests_calculate_price_stability {
    use super::*;
    use rust_decimal_macros::dec;
    use tracing::info;
    use crate::utils::logger::setup_logger;

    fn test_stability(min: Decimal, max: Decimal) {
        let stability = calculate_price_stability(min, max);
        info!("Min: {}, Max: {}, Stability: {}", min, max, stability);
    }

    #[test]
    fn test_price_stability() {
        setup_logger();
        test_stability(dec!(0), dec!(0));     // Both prices zero
        test_stability(dec!(100), dec!(100)); // No price change
        test_stability(dec!(90), dec!(110));  // Normal case
        test_stability(dec!(0), dec!(100));   // Extreme change
        test_stability(dec!(-10), dec!(10));  // Negative price
        test_stability(dec!(110), dec!(90));  // Min greater than max
    }
}

#[cfg(test)]
mod tests_calculate_liquidity_efficiency {
    use super::*;
    use rust_decimal_macros::dec;
    use tracing::info;
    use crate::utils::logger::setup_logger;

    fn test_efficiency(change: Decimal) {
        let efficiency = calculate_liquidity_efficiency(change);
        info!("Change: {}, Efficiency: {}", change, efficiency);
    }

    #[test]
    fn test_liquidity_efficiency() {
        setup_logger();
        test_efficiency(dec!(0));       // Normal case: no change
        test_efficiency(dec!(0.5));     // Normal case: positive change
        test_efficiency(dec!(-0.5));    // Normal case: negative change
        test_efficiency(dec!(-1));      // Extreme case: denominator would be zero
        test_efficiency(dec!(100));     // Extreme case: very large positive change
        test_efficiency(dec!(-0.99));   // Extreme case: negative change close to -1
    }
}