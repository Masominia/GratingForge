use gratingforge::math::numeric::trapezoid;
use gratingforge::optics::emission::{emit, Apodisation};

fn power(e: &gratingforge::optics::emission::Emission) -> f64 {
    let squares: Vec<f64> = e.amplitude.iter().map(|a| a * a).collect();
    trapezoid(&squares, e.step)
}

#[test]
fn uniform_grating_decays_exponentially() {
    let e = emit(20.0, 0.2, &Apodisation::Uniform, 2401);
    assert!((e.radiated - (1.0 - (-4.0f64).exp())).abs() < 1e-12);
    let i = 600; // x = 5 um
    assert!((e.amplitude[i] - (0.2f64 * (-1.0f64).exp()).sqrt()).abs() < 1e-9);
}

#[test]
fn emitted_power_equals_radiated_fraction() {
    for apodisation in [Apodisation::Uniform, Apodisation::Linear { from: 0.02, to: 0.4 }, Apodisation::Gaussian { radius: 5.0, centre: 12.0 }] {
        let e = emit(24.0, 0.5, &apodisation, 4001);
        assert!((power(&e) - e.radiated).abs() < 1e-5, "{apodisation:?}");
        assert!(e.radiated > 0.0 && e.radiated < 1.0);
    }
}

#[test]
fn gaussian_apodisation_emits_a_gaussian() {
    let (radius, centre) = (5.0, 15.0);
    let e = emit(30.0, 10.0, &Apodisation::Gaussian { radius, centre }, 3001);
    let peak = e.amplitude.iter().cloned().fold(0.0, f64::max);
    for x in [9.0, 12.0, 15.0, 18.0] {
        let i = (x / e.step).round() as usize;
        let expected = peak * (-(x - centre).powi(2) / (radius * radius)).exp();
        assert!((e.amplitude[i] - expected).abs() < 2e-3 * peak, "{x}");
    }
    assert!((e.radiated - 0.999).abs() < 1e-3);
}

#[test]
fn the_leakage_cap_is_respected() {
    let e = emit(30.0, 0.3, &Apodisation::Gaussian { radius: 5.0, centre: 15.0 }, 3001);
    assert!(e.leakage.iter().all(|l| *l <= 0.3 + 1e-12));
    assert!(e.leakage.iter().any(|l| (*l - 0.3).abs() < 1e-12));
}

#[test]
fn a_rising_ramp_moves_the_emission_downstream() {
    let centroid = |e: &gratingforge::optics::emission::Emission| {
        let (mut num, mut den) = (0.0, 0.0);
        for i in 0..e.len() {
            num += e.position(i) * e.amplitude[i].powi(2);
            den += e.amplitude[i].powi(2);
        }
        num / den
    };
    let up = emit(25.0, 0.0, &Apodisation::Linear { from: 0.03, to: 0.3 }, 2401);
    let down = emit(25.0, 0.0, &Apodisation::Linear { from: 0.3, to: 0.03 }, 2401);
    assert!(centroid(&up) > centroid(&down) + 3.0);
}
