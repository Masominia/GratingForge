//! Overlap of a uniform grating with a fibre mode against the leakage rate: the 80% limit.
//!
//!     cargo run --release --example leakage_sweep

use gratingforge::optics::emission::{emit, Apodisation};
use gratingforge::optics::fibre::{best_position, Fibre};

fn main() {
    let fibre = Fibre { mfd: 10.4, angle: 10.0 };
    let sin_out = 10f64.to_radians().sin();
    println!("{:>12} {:>10} {:>12}", "leak, 1/um", "overlap", "position, um");
    for k in 1..=20 {
        let leak = 0.04 * k as f64;
        let e = emit(60.0, leak, &Apodisation::Uniform, 6001);
        let (x, c) = best_position(&e, &fibre, sin_out, 1.444, 1.55);
        println!("{leak:>12.2} {:>10.4} {x:>12.2}", c / e.radiated);
    }
}
