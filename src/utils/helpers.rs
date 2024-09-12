/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

use rand::Rng;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Function that returns a random number between 1 and X (where X is of type Decimal)
pub(crate) fn random_decimal(x: Decimal) -> Decimal {
    if x <= dec!(1) {
        return dec!(1);
    }
    let mut rng = rand::thread_rng();
    let x_f64 = x.to_f64().expect("Error converting Decimal to f64");
    let random_f64 = rng.gen_range(1.0..x_f64);
    Decimal::from_f64(random_f64).expect("Error converting f64 to Decimal")
}

pub fn format_float(value: f64, decimals: usize) -> String {
    format!("{:.1$}", value, decimals)
}

#[cfg(test)]
mod tests_format_float {
    use super::*;

    #[test]
    fn test_format_float_zero_decimals() {
        let result = format_float(1234.5678, 0);
        assert_eq!(result, "1235"); // Rounded without decimals
    }

    #[test]
    fn test_format_float_two_decimals() {
        let result = format_float(1234.5678, 2);
        assert_eq!(result, "1234.57"); // Rounded to two decimal places
    }

    #[test]
    fn test_format_float_no_significant_change() {
        let result = format_float(1234.5678, 4);
        assert_eq!(result, "1234.5678"); // Matches the input
    }

    #[test]
    fn test_format_float_rounding_up() {
        let result = format_float(0.99999, 2);
        assert_eq!(result, "1.00"); // Rounded up
    }

    #[test]
    fn test_format_float_rounding_down() {
        let result = format_float(0.12345, 3);
        assert_eq!(result, "0.123"); // Rounded down
    }
}
