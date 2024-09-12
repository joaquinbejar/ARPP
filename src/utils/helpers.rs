/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/9/24
******************************************************************************/

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