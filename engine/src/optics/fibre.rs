//! Overlap of the emitted field with a tilted Gaussian fibre mode.

use super::emission::Emission;
use crate::math::numeric::golden_max;
use crate::C64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fibre {
    /// Mode field diameter (1/e^2 intensity), micrometres.
    pub mfd: f64,
    /// Tilt from the surface normal, degrees, in the medium above the grating.
    pub angle: f64,
}

impl Fibre {
    /// 1/e^2 radius of the mode's footprint on the grating plane: wider than the mode by 1 / cos(angle).
    pub fn footprint(&self) -> f64 {
        0.5 * self.mfd / self.angle.to_radians().cos()
    }
}

/// Fraction of the guided power coupled into the fibre: the squared overlap of the emitted field with the
/// normalised fibre mode. `sin_emission` and the fibre angle refer to the same medium of index `n_out`.
/// The emitted field already carries the radiated fraction, so this is overlap times radiated fraction.
pub fn coupled_fraction(emission: &Emission, fibre: &Fibre, position: f64, sin_emission: f64, n_out: f64, lambda: f64) -> f64 {
    let w = fibre.footprint();
    let dk = 2.0 * std::f64::consts::PI / lambda * n_out * (sin_emission - fibre.angle.to_radians().sin());
    let mut sum = C64::ZERO;
    for (i, a) in emission.amplitude.iter().enumerate() {
        let x = emission.position(i);
        let weight = if i == 0 || i + 1 == emission.len() { 0.5 } else { 1.0 };
        let g = (-(x - position).powi(2) / (w * w)).exp();
        sum += C64::polar(a * g * weight, dk * x);
    }
    // integral of g^2 over the whole line: w sqrt(pi / 2)
    let norm = w * (std::f64::consts::PI / 2.0).sqrt();
    sum.norm_sqr() * emission.step * emission.step / norm
}

/// The fibre position along the grating that maximises the coupling, and the coupling there.
pub fn best_position(emission: &Emission, fibre: &Fibre, sin_emission: f64, n_out: f64, lambda: f64) -> (f64, f64) {
    let length = emission.step * (emission.len() - 1) as f64;
    // coarse scan first: the coupling can have a shoulder for strongly apodised gratings
    let coarse = (0..=60).map(|i| -0.25 * length + 1.5 * length * i as f64 / 60.0).map(|x| (x, coupled_fraction(emission, fibre, x, sin_emission, n_out, lambda))).fold((0.0, -1.0), |best, c| if c.1 > best.1 { c } else { best });
    let span = 1.5 * length / 60.0;
    golden_max(|x| coupled_fraction(emission, fibre, x, sin_emission, n_out, lambda), coarse.0 - span, coarse.0 + span)
}
