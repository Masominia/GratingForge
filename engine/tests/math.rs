use gratingforge::math::numeric::{bisect, golden_max, interpolate, trapezoid};
use gratingforge::C64;

#[test]
fn bisect_finds_a_root_and_rejects_a_bad_bracket() {
    let r = bisect(|x| x * x - 2.0, 0.0, 2.0).unwrap();
    assert!((r - 2f64.sqrt()).abs() < 1e-12);
    assert!(bisect(|x| x * x + 1.0, 0.0, 2.0).is_none());
}

#[test]
fn golden_section_finds_the_maximum() {
    let (x, v) = golden_max(|x| 3.0 - (x - 0.7).powi(2), -2.0, 2.0);
    assert!((x - 0.7).abs() < 1e-6 && (v - 3.0).abs() < 1e-10);
}

#[test]
fn trapezoid_is_exact_for_a_straight_line() {
    let values: Vec<f64> = (0..11).map(|i| 2.0 * i as f64 * 0.1 + 1.0).collect();
    assert!((trapezoid(&values, 0.1) - 2.0).abs() < 1e-12);
}

#[test]
fn interpolation_is_linear_and_clamped() {
    let values = [0.0, 1.0, 4.0];
    assert!((interpolate(&values, 10.0, 2.0, 11.0) - 0.5).abs() < 1e-12);
    assert!((interpolate(&values, 10.0, 2.0, 13.0) - 2.5).abs() < 1e-12);
    assert_eq!(interpolate(&values, 10.0, 2.0, 0.0), 0.0);
    assert_eq!(interpolate(&values, 10.0, 2.0, 99.0), 4.0);
}

#[test]
fn complex_polar_form() {
    let z = C64::polar(2.0, std::f64::consts::FRAC_PI_2);
    assert!(z.re.abs() < 1e-12 && (z.im - 2.0).abs() < 1e-12);
    assert!((z.norm_sqr() - 4.0).abs() < 1e-12);
}
