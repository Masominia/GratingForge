//! Guided modes of a three-layer slab: cladding / core / substrate.

use crate::math::numeric::bisect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polarisation {
    Te,
    Tm,
}

impl Polarisation {
    pub fn parse(text: &str) -> Option<Polarisation> {
        match text {
            "te" => Some(Polarisation::Te),
            "tm" => Some(Polarisation::Tm),
            _ => None,
        }
    }
}

/// Dispersion function whose root is the effective index of mode `order`:
/// kappa t - atan(g_c / kappa) - atan(g_s / kappa) - order pi, with the TM weights n_core^2 / n^2 on the decay rates.
fn dispersion(n_eff: f64, lambda: f64, thickness: f64, n_core: f64, n_clad: f64, n_sub: f64, pol: Polarisation, order: u32) -> f64 {
    let k = 2.0 * std::f64::consts::PI / lambda;
    let kappa = k * (n_core * n_core - n_eff * n_eff).max(0.0).sqrt();
    let (gc, gs) = (k * (n_eff * n_eff - n_clad * n_clad).max(0.0).sqrt(), k * (n_eff * n_eff - n_sub * n_sub).max(0.0).sqrt());
    let (wc, ws) = match pol {
        Polarisation::Te => (1.0, 1.0),
        Polarisation::Tm => ((n_core / n_clad).powi(2), (n_core / n_sub).powi(2)),
    };
    kappa * thickness - (wc * gc).atan2(kappa) - (ws * gs).atan2(kappa) - order as f64 * std::f64::consts::PI
}

/// Effective index of a guided mode, or None if the mode is cut off (or the core is not the highest index).
pub fn effective_index(lambda: f64, thickness: f64, n_core: f64, n_clad: f64, n_sub: f64, pol: Polarisation, order: u32) -> Option<f64> {
    let floor = n_clad.max(n_sub);
    if !(n_core > floor) || !(thickness > 0.0) {
        return None;
    }
    let f = |n: f64| dispersion(n, lambda, thickness, n_core, n_clad, n_sub, pol, order);
    // the function falls monotonically from the cut-off value at `floor` to a negative value at n_core
    let (lo, hi) = (floor + 1e-12, n_core - 1e-12);
    if f(lo) <= 0.0 {
        return None;
    }
    bisect(f, lo, hi)
}

/// Number of guided modes of one polarisation.
pub fn mode_count(lambda: f64, thickness: f64, n_core: f64, n_clad: f64, n_sub: f64, pol: Polarisation) -> u32 {
    (0..64).take_while(|m| effective_index(lambda, thickness, n_core, n_clad, n_sub, pol, *m).is_some()).count() as u32
}
