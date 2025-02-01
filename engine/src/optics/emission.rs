//! The field radiated along the grating for a given leakage profile.

use crate::math::numeric::trapezoid;

#[derive(Clone, Debug, PartialEq)]
pub enum Apodisation {
    /// Constant leakage: an exponentially decaying field.
    Uniform,
    /// Leakage rising or falling linearly from `from` to `to` (per micrometre, in power) along the grating.
    Linear { from: f64, to: f64 },
    /// Leakage chosen so that the emitted intensity follows a Gaussian of 1/e^2 radius `radius` centred at `centre`,
    /// never exceeding the maximum leakage the etch can provide.
    Gaussian { radius: f64, centre: f64 },
}

#[derive(Clone, Debug)]
pub struct Emission {
    pub step: f64,
    /// Field amplitude at each sample, normalised so that the integral of its square is the radiated fraction.
    pub amplitude: Vec<f64>,
    /// Leakage rate (power, per micrometre) at each sample.
    pub leakage: Vec<f64>,
    /// Fraction of the guided power radiated over the length.
    pub radiated: f64,
}

impl Emission {
    pub fn len(&self) -> usize {
        self.amplitude.len()
    }

    pub fn is_empty(&self) -> bool {
        self.amplitude.is_empty()
    }

    pub fn position(&self, i: usize) -> f64 {
        i as f64 * self.step
    }
}

/// Leakage rate needed at x so that the emitted intensity is a target profile g(x)^2 (unit total power)
/// scaled to radiate a fraction F of the guided power in all: leakage = F g^2 / (1 - F * integral of g^2 up to x),
/// with g^2 normalised to unit integral over the grating.
fn gaussian_leakage(x: &[f64], step: f64, radius: f64, centre: f64, cap: f64) -> Vec<f64> {
    let norm = (2.0 / std::f64::consts::PI).sqrt() / radius;
    let g2: Vec<f64> = x.iter().map(|xi| norm * (-2.0 * (xi - centre).powi(2) / (radius * radius)).exp()).collect();
    // share of the target profile that falls on the grating; the design radiates 99.9% of the guided power
    let inside = trapezoid(&g2, step).max(1e-12);
    let fraction = 0.999;
    let mut out = Vec::with_capacity(x.len());
    let mut cumulative = 0.0;
    for i in 0..x.len() {
        if i > 0 {
            cumulative += 0.5 * (g2[i] + g2[i - 1]) * step;
        }
        let remaining = (1.0 - fraction * cumulative / inside).max(1e-9);
        out.push((fraction * g2[i] / inside / remaining).min(cap));
    }
    out
}

/// The emitted field of a grating of the given length. `leak` is the power leakage rate per micrometre
/// (twice the amplitude decay rate): the constant value for a uniform grating, the upper limit for a Gaussian one.
pub fn emit(length: f64, leak: f64, apodisation: &Apodisation, samples: usize) -> Emission {
    let n = samples.max(3);
    let step = length / (n - 1) as f64;
    let x: Vec<f64> = (0..n).map(|i| i as f64 * step).collect();
    let leakage: Vec<f64> = match apodisation {
        Apodisation::Uniform => vec![leak; n],
        Apodisation::Linear { from, to } => x.iter().map(|xi| (from + (to - from) * xi / length).max(0.0)).collect(),
        Apodisation::Gaussian { radius, centre } => gaussian_leakage(&x, step, *radius, *centre, leak),
    };
    // guided power P(x) = exp(-integral of the leakage); emitted intensity = leakage * P
    let mut amplitude = Vec::with_capacity(n);
    let mut exponent = 0.0;
    for i in 0..n {
        if i > 0 {
            exponent += 0.5 * (leakage[i] + leakage[i - 1]) * step;
        }
        amplitude.push((leakage[i] * (-exponent).exp()).sqrt());
    }
    Emission { step, amplitude, leakage, radiated: 1.0 - (-exponent).exp() }
}
