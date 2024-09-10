/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 4/9/24
******************************************************************************/

use criterion::criterion_main;

mod bench;

criterion_main! {
    bench::benches,
}
