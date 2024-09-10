/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 10/9/24
 ******************************************************************************/

pub fn format_float(value: f64, decimals: usize) -> String {
    // TODO: Implement float formatting helper
    format!("{:.1$}", value, decimals)
}
