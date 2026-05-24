use gratingforge::optics::emission::{emit, Apodisation};
use gratingforge::optics::fibre::{best_position, coupled_fraction, Fibre};

#[test]
fn footprint_widens_with_tilt() {
    assert!((Fibre { mfd: 10.4, angle: 0.0 }.footprint() - 5.2).abs() < 1e-12);
    assert!((Fibre { mfd: 10.4, angle: 60.0 }.footprint() - 10.4).abs() < 1e-9);
}

#[test]
fn a_matched_gaussian_couples_almost_completely() {
    let fibre = Fibre { mfd: 10.4, angle: 10.0 };
    let e = emit(40.0, 20.0, &Apodisation::Gaussian { radius: fibre.footprint(), centre: 20.0 }, 4001);
    let s = 10f64.to_radians().sin();
    let c = coupled_fraction(&e, &fibre, 20.0, s, 1.444, 1.55);
    assert!(c > 0.997 && c <= 0.9991, "{c}");
}

#[test]
fn uniform_grating_cannot_exceed_the_exponential_limit() {
    let fibre = Fibre { mfd: 10.4, angle: 0.0 };
    let mut best = 0.0f64;
    for k in 1..60 {
        let e = emit(80.0, 0.01 * k as f64, &Apodisation::Uniform, 8001);
        best = best.max(best_position(&e, &fibre, 0.0, 1.0, 1.55).1);
    }
    assert!((best - 0.801).abs() < 1e-3, "{best}");
}

#[test]
fn best_position_beats_its_neighbours() {
    let fibre = Fibre { mfd: 10.4, angle: 10.0 };
    let e = emit(30.0, 0.14, &Apodisation::Uniform, 3001);
    let s = 10f64.to_radians().sin();
    let (x, c) = best_position(&e, &fibre, s, 1.444, 1.55);
    for dx in [-1.0, -0.1, 0.1, 1.0] {
        assert!(coupled_fraction(&e, &fibre, x + dx, s, 1.444, 1.55) < c);
    }
}

#[test]
fn angular_mismatch_follows_the_gaussian_law() {
    // matched Gaussian beams tilted against each other: exp(-(k n w dsin)^2 / 4)
    let fibre = Fibre { mfd: 10.4, angle: 10.0 };
    let w = fibre.footprint();
    let e = emit(50.0, 20.0, &Apodisation::Gaussian { radius: w, centre: 25.0 }, 5001);
    let s0 = 10f64.to_radians().sin();
    let aligned = coupled_fraction(&e, &fibre, 25.0, s0, 1.444, 1.55);
    for ds in [0.01, 0.02, 0.04] {
        let dk = 2.0 * std::f64::consts::PI / 1.55 * 1.444 * ds;
        let expected = aligned * (-(dk * w).powi(2) / 4.0).exp();
        let got = coupled_fraction(&e, &fibre, 25.0, s0 + ds, 1.444, 1.55);
        assert!((got / expected - 1.0).abs() < 5e-3, "{ds}: {got} {expected}");
    }
}
