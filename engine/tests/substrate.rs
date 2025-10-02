use gratingforge::optics::substrate::{fresnel_te, thickness_period, upward_fraction};

#[test]
fn fresnel_at_normal_incidence() {
    assert!((fresnel_te(1.444, 3.476, 0.0) - (1.444 - 3.476) / (1.444 + 3.476)).abs() < 1e-12);
    assert_eq!(fresnel_te(3.476, 1.0, 1.0), -1.0);
}

#[test]
fn upward_fraction_stays_a_fraction() {
    for k in 0..200 {
        let up = upward_fraction(0.5, 1.444, 0.17, 1.444, 1.0 + 0.01 * k as f64, 3.476, 1.55);
        assert!(up > 0.0 && up < 1.0);
    }
}

#[test]
fn oxide_thickness_swings_the_upward_fraction_around_the_intrinsic_value() {
    let values: Vec<f64> = (0..400).map(|k| upward_fraction(0.5, 1.444, 0.17, 1.444, 1.0 + 0.004 * k as f64, 3.476, 1.55)).collect();
    let (lo, hi) = values.iter().fold((1.0f64, 0.0f64), |(l, h), v| (l.min(*v), h.max(*v)));
    assert!(lo < 0.35 && hi > 0.65, "{lo} {hi}");
}

#[test]
fn the_swing_repeats_with_the_thickness_period() {
    let p = thickness_period(1.444, 0.17, 1.444, 1.55);
    assert!((p - 1.55 / (2.0 * 1.444 * (1.0f64 - 0.17 * 0.17).sqrt())).abs() < 1e-12);
    let a = upward_fraction(0.4, 1.444, 0.17, 1.444, 1.7, 3.476, 1.55);
    let b = upward_fraction(0.4, 1.444, 0.17, 1.444, 1.7 + p, 3.476, 1.55);
    assert!((a - b).abs() < 1e-10);
}

#[test]
fn a_perfectly_directional_grating_ignores_the_substrate() {
    assert!((upward_fraction(1.0, 1.444, 0.17, 1.444, 2.0, 3.476, 1.55) - 1.0).abs() < 1e-12);
}

#[test]
fn an_index_matched_substrate_changes_nothing() {
    assert!((upward_fraction(0.42, 1.444, 0.17, 1.444, 2.0, 1.444, 1.55) - 0.42).abs() < 1e-12);
}
