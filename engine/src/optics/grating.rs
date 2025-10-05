//! The grating equation.

/// Mean effective index of a grating whose period has a fraction `fill` of unetched slab.
pub fn mean_index(n_full: f64, n_etched: f64, fill: f64) -> f64 {
    fill * n_full + (1.0 - fill) * n_etched
}

/// Sine of the emission angle in a medium of index `n_out` for diffraction order `order` (1 is the usual one).
/// A magnitude above one means the order does not radiate into that medium.
pub fn sin_angle(n_grating: f64, lambda: f64, period: f64, n_out: f64, order: i32) -> f64 {
    (n_grating - order as f64 * lambda / period) / n_out
}

/// Emission angle in degrees, positive in the direction of propagation; None if the order is evanescent.
pub fn angle(n_grating: f64, lambda: f64, period: f64, n_out: f64, order: i32) -> Option<f64> {
    let s = sin_angle(n_grating, lambda, period, n_out, order);
    if s.abs() <= 1.0 {
        Some(s.asin().to_degrees())
    } else {
        None
    }
}

/// Period that sends the first order to `angle_deg` in a medium of index `n_out`.
pub fn period_for_angle(n_grating: f64, lambda: f64, n_out: f64, angle_deg: f64) -> f64 {
    lambda / (n_grating - n_out * angle_deg.to_radians().sin())
}

/// Orders that radiate into a medium of index `n_out`.
pub fn radiating_orders(n_grating: f64, lambda: f64, period: f64, n_out: f64) -> Vec<i32> {
    (1..=20).filter(|m| sin_angle(n_grating, lambda, period, n_out, *m).abs() <= 1.0).collect()
}

/// Wavelength at which the second order reflects the guided mode back on itself (emission exactly vertical).
pub fn second_order_wavelength(n_grating: f64, period: f64) -> f64 {
    n_grating * period
}
