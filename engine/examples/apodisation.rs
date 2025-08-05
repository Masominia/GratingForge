//! Uniform against Gaussian-apodised emission for increasing limits on the leakage rate.
//!
//!     cargo run --release --example apodisation

use gratingforge::optics::emission::{emit, Apodisation};
use gratingforge::optics::fibre::{best_position, Fibre};

fn main() {
    let fibre = Fibre { mfd: 10.4, angle: 10.0 };
    let sin_out = 10f64.to_radians().sin();
    println!("{:>14} {:>10} {:>10}", "max leak, 1/um", "radiated", "overlap");
    for cap in [0.1, 0.2, 0.4, 0.8, 1.6, 3.2] {
        let e = emit(30.0, cap, &Apodisation::Gaussian { radius: fibre.footprint(), centre: 15.0 }, 3001);
        let (_, c) = best_position(&e, &fibre, sin_out, 1.444, 1.55);
        println!("{cap:>14.1} {:>10.4} {:>10.4}", e.radiated, c / e.radiated);
    }
}
