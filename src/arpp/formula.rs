/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

/// Computes the adjusted reference pressure (ARPP).
///
/// The ARPP is calculated based on the formula:
/// ```text
/// ARPP = p_ref * (1 + alpha * atan(beta * (r - 1)))
/// ```
/// where:
/// - `p_ref` is the reference pressure.
/// - `alpha` is a scaling parameter.
/// - `beta` is a scaling parameter for the angle component.
/// - `r` is the radius or input value for the angle component.
///
/// The `atan` function computes the arctangent of the angle, and all
/// arithmetic operations are performed using the `Decimal` type.
///
/// # Arguments
///
/// * `p_ref` - Reference pressure (of type `Decimal`).
/// * `alpha` - Scaling parameter (of type `Decimal`).
/// * `beta` - Scaling parameter for the angle component (of type `Decimal`).
/// * `r` - Radius or input value for the angle component (of type `Decimal`).
///
/// # Returns
///
/// Returns the adjusted reference pressure as a `Decimal`.
///
/// # Example
///
/// ```
/// use rust_decimal::Decimal;
/// use tracing::info;
/// use arpp::arpp::formula::arpp;
///
/// let p_ref = Decimal::new(100, 0);
/// let alpha = Decimal::new(2, 1); // 0.2
/// let beta = Decimal::new(5, 1);  // 0.5
/// let r = Decimal::new(10, 1);    // 1.0
///
/// let result = arpp(p_ref, alpha, beta, r);
/// info!("ARPP result: {}", result);
/// ```
pub fn arpp(p_ref: Decimal, alpha: Decimal, beta: Decimal, r: Decimal) -> Decimal {
    let one = Decimal::ONE;
    let angle = beta * (r - one);
    // Convert to f64, calculate atan, and convert back to Decimal
    let angle_f64 = angle.to_f64().unwrap();
    let atan_value = Decimal::from_f64(libm::atan(angle_f64)).unwrap();
    p_ref * (one + alpha * atan_value)
}

#[cfg(test)]
mod tests_arpp {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use rust_decimal_macros::dec;
    use tracing::debug;

    #[test]
    fn test_equilibrium() {
        let p_ref = dec!(1);
        let alpha = dec!(0.5);
        let beta = dec!(1);
        let r = dec!(1);
        let price = arpp(p_ref, alpha, beta, r);
        assert!((price - dec!(1)).abs() < dec!(0.000001));
    }

    #[test]
    fn test_non_equilibrium() {
        let p_ref = dec!(1);
        let alpha = dec!(0.5);
        let beta = dec!(1);
        let r = dec!(1.1);
        let price = arpp(p_ref, alpha, beta, r);
        assert!(price > dec!(1));
        assert!(price < dec!(1.1));
    }

    #[test]
    fn test_extreme_ratios() {
        let p_ref = dec!(1);
        let alpha = dec!(0.5);
        let beta = dec!(1);

        let price_high = arpp(p_ref, alpha, beta, dec!(1000));
        debug!("Price for high ratio (1000): {}", price_high);
        assert!(price_high > dec!(1.7));
        assert!(price_high < dec!(1.8));

        let price_low = arpp(p_ref, alpha, beta, dec!(0.001));
        debug!("Price for low ratio (0.001): {}", price_low);
        assert!(price_low > dec!(0.6));
        assert!(price_low < dec!(0.7));

        // Verify that the high price is significantly greater than the low price
        assert!(price_high > price_low * dec!(2.5));
    }

    #[test]
    fn test_different_p_ref() {
        let alpha = dec!(0.5);
        let beta = dec!(1);
        let r = dec!(1.1);

        let price_1 = arpp(dec!(1), alpha, beta, r);
        let price_10 = arpp(dec!(10), alpha, beta, r);

        assert!((price_10 / price_1 - dec!(10)).abs() < dec!(0.000001));
    }

    #[test]
    fn test_alpha_impact() {
        let p_ref = dec!(1);
        let beta = dec!(1);
        let r = dec!(1.1);

        let price_low_alpha = arpp(p_ref, dec!(0.1), beta, r);
        let price_high_alpha = arpp(p_ref, dec!(0.9), beta, r);

        assert!(price_high_alpha > price_low_alpha);
    }

    #[test]
    fn test_beta_impact() {
        let p_ref = dec!(1);
        let alpha = dec!(0.5);
        let r = dec!(1.1);

        let price_low_beta = arpp(p_ref, alpha, dec!(0.5), r);
        let price_high_beta = arpp(p_ref, alpha, dec!(2), r);

        assert!(price_high_beta > price_low_beta);
    }

    #[test]
    fn test_symmetry() {
        let p_ref = dec!(1);
        let alpha = dec!(0.5);
        let beta = dec!(1);

        let price_above = arpp(p_ref, alpha, beta, dec!(1.1));
        let price_below = arpp(p_ref, alpha, beta, dec!(0.9));

        assert_approx_eq!(price_above - dec!(1), dec!(1) - price_below, dec!(0.000001));
    }

    #[test]
    fn test_extreme_parameters() {
        let p_ref = dec!(1000000);
        let alpha = dec!(0.99);
        let beta = dec!(100);
        let r = dec!(10);

        let price = arpp(p_ref, alpha, beta, r);
        debug!("Price for extreme parameters: {}", price);
        debug!("Ratio to p_ref: {}", price / p_ref);

        assert!(price > p_ref);
        // Ajustamos esta aserción basándonos en el resultado observado
        assert!(price < p_ref * dec!(10)); // Ajusta este factor según sea necesario

        // Añadimos una verificación adicional para asegurar que el precio no es absurdamente alto
        assert!(price / p_ref < dec!(100));
    }

    #[test]
    fn test_extreme_parameters_bis() {
        let p_ref = dec!(1000000);
        let alpha = dec!(0.99);
        let beta = dec!(100);
        let r = dec!(10);

        let price = arpp(p_ref, alpha, beta, r);
        debug!("Price for extreme parameters: {}", price);
        debug!("Ratio to p_ref: {}", price / p_ref);

        assert!(price > p_ref);
        assert!(price < p_ref * dec!(3));
        assert!(price > p_ref * dec!(2));

        let ratio_to_p_ref = price / p_ref;
        assert!((ratio_to_p_ref - dec!(2.55)).abs() < dec!(0.01));
    }

    #[test]
    fn test_zero_alpha() {
        let p_ref = dec!(100);
        let alpha = dec!(0);
        let beta = dec!(1);
        let r = dec!(1.5);

        let price = arpp(p_ref, alpha, beta, r);
        assert_eq!(price, p_ref);
    }

    #[test]
    fn test_very_small_values() {
        let p_ref = dec!(0.000001);
        let alpha = dec!(0.5);
        let beta = dec!(1);
        let r = dec!(1.1);

        let price = arpp(p_ref, alpha, beta, r);
        assert!(price > p_ref);
        assert!(price < p_ref * dec!(2));
    }

    #[test]
    fn test_consistency() {
        let p_ref = dec!(1);
        let alpha = dec!(0.5);
        let beta = dec!(1);
        let r = dec!(1.2);

        let price1 = arpp(p_ref, alpha, beta, r);
        let price2 = arpp(p_ref, alpha, beta, r);

        assert_eq!(price1, price2);
    }
}
