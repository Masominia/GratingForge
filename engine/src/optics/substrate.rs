//! Two-beam estimate of how the reflection from the substrate changes the upward fraction.

use crate::C64;

/// Fresnel amplitude reflection for TE light going from index n1 to n2 at angle `theta1` (radians) in medium 1.
/// Returns -1 beyond the critical angle.
pub fn fresnel_te(n1: f64, n2: f64, theta1: f64) -> f64 {
    let s2 = n1 * theta1.sin() / n2;
    if s2.abs() >= 1.0 {
        return -1.0;
    }
    let (c1, c2) = (theta1.cos(), (1.0 - s2 * s2).sqrt());
    (n1 * c1 - n2 * c2) / (n1 * c1 + n2 * c2)
}

/// Upward fraction when the downward wave is partly reflected by the substrate and returns in phase or out of phase.
///
/// `intrinsic_up` is the upward fraction of the grating alone. The downward wave of amplitude sqrt(1 - up) crosses
/// the buried oxide (index `n_box`, thickness `t_box`) at the refracted angle, reflects with the Fresnel coefficient
/// of the oxide-substrate interface and adds to the upward wave. The result is the upward power over the upward
/// power plus what is transmitted into the substrate. Multiple reflections are ignored.
pub fn upward_fraction(intrinsic_up: f64, n_out: f64, sin_out: f64, n_box: f64, t_box: f64, n_sub: f64, lambda: f64) -> f64 {
    let sin_box = n_out * sin_out / n_box;
    if sin_box.abs() >= 1.0 {
        return intrinsic_up;
    }
    let theta_box = sin_box.asin();
    let r = fresnel_te(n_box, n_sub, theta_box);
    let phase = 2.0 * 2.0 * std::f64::consts::PI / lambda * n_box * t_box * theta_box.cos();
    let (up, down) = (intrinsic_up.sqrt(), (1.0 - intrinsic_up).sqrt());
    let total_up = (C64::new(up, 0.0) + C64::polar(down * r, phase)).norm_sqr();
    let lost = (1.0 - intrinsic_up) * (1.0 - r * r);
    total_up / (total_up + lost)
}

/// Change of oxide thickness that brings the interference back to the same state: lambda / (2 n_box cos(theta_box)).
pub fn thickness_period(n_out: f64, sin_out: f64, n_box: f64, lambda: f64) -> f64 {
    let sin_box = n_out * sin_out / n_box;
    lambda / (2.0 * n_box * (1.0 - sin_box * sin_box).max(0.0).sqrt())
}
