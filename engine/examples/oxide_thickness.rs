//! Upward fraction against buried-oxide thickness for three intrinsic directionalities.
//!
//!     cargo run --release --example oxide_thickness

use gratingforge::optics::substrate::{thickness_period, upward_fraction};

fn main() {
    let sin_out = 10f64.to_radians().sin();
    println!("interference period {:.4} um", thickness_period(1.444, sin_out, 1.444, 1.55));
    println!("{:>8} {:>8} {:>8} {:>8}", "box, um", "up 0.5", "up 0.7", "up 0.9");
    for k in 0..=20 {
        let t = 1.0 + 0.1 * k as f64;
        let row: Vec<String> = [0.5, 0.7, 0.9].iter().map(|u| format!("{:>8.4}", upward_fraction(*u, 1.444, sin_out, 1.444, t, 3.476, 1.55))).collect();
        println!("{t:>8.1} {}", row.join(" "));
    }
}
