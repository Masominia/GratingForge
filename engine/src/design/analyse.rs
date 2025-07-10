//! Everything computed for one design.

use super::spec::{ApodisationSpec, PeriodSpec, Spec};
use crate::optics::emission::{emit, Apodisation, Emission};
use crate::optics::fibre::{best_position, coupled_fraction, Fibre};
use crate::optics::grating;
use crate::optics::slab::effective_index;
use crate::optics::substrate::upward_fraction;

#[derive(Clone, Copy, Debug)]
pub struct Indices {
    pub full: f64,
    pub etched: f64,
    pub grating: f64,
    pub clad: f64,
}

#[derive(Debug)]
pub struct Analysis {
    pub indices: Indices,
    /// Group index of the mean grating index: n - lambda dn/dlambda.
    pub group_index: f64,
    pub period: f64,
    /// Emission angle in the cladding at the design wavelength, degrees.
    pub angle: f64,
    pub orders: Vec<i32>,
    pub second_order_wavelength: f64,
    pub emission: Emission,
    /// Upward fraction at the design wavelength, substrate included if switched on.
    pub up: f64,
    pub fibre: Fibre,
    pub position: f64,
    /// Overlap of the emitted field with the fibre mode: coupled power over radiated power.
    pub overlap: f64,
    /// Guided power to fibre: up times radiated times overlap.
    pub efficiency: f64,
    pub spectrum: Vec<(f64, f64)>,
    pub peak_wavelength: f64,
    pub peak_efficiency: f64,
    /// Full widths in nanometres at 1 and 3 dB below the peak; NaN if the sweep does not contain them.
    pub bandwidth_1db: f64,
    pub bandwidth_3db: f64,
    /// Change of emission angle with wavelength, degrees per nanometre.
    pub dtheta_dlambda: f64,
    /// 1 dB width predicted for a Gaussian emitted field matched to the fibre, nanometres.
    pub bandwidth_1db_formula: f64,
}

/// Effective indices at a wavelength. Returns an error if the unetched slab guides no mode.
pub fn indices(spec: &Spec, lambda: f64) -> Result<Indices, String> {
    let s = &spec.stack;
    let (nc, ncl, nb) = (s.n_core.index(lambda), s.n_clad.index(lambda), s.n_box.index(lambda));
    let full = effective_index(lambda, s.core, nc, ncl, nb, spec.polarisation, 0).ok_or("the slab guides no mode at this wavelength")?;
    let remaining = s.core - spec.etch;
    // an etched section too thin to guide is taken at the index of the higher of its two neighbours
    let etched = if remaining > 1e-9 { effective_index(lambda, remaining, nc, ncl, nb, spec.polarisation, 0).unwrap_or(ncl.max(nb)) } else { ncl.max(nb) };
    Ok(Indices { full, etched, grating: grating::mean_index(full, etched, spec.fill), clad: ncl })
}

fn width_below_peak(spectrum: &[(f64, f64)], peak_index: usize, drop_db: f64) -> f64 {
    let level = spectrum[peak_index].1 * 10f64.powf(-drop_db / 10.0);
    let cross = |range: &mut dyn Iterator<Item = usize>| -> f64 {
        let mut previous = peak_index;
        for i in range {
            if spectrum[i].1 <= level {
                let (a, b) = (spectrum[previous], spectrum[i]);
                return a.0 + (a.1 - level) / (a.1 - b.1) * (b.0 - a.0);
            }
            previous = i;
        }
        f64::NAN
    };
    let right = cross(&mut (peak_index + 1..spectrum.len()));
    let left = cross(&mut (0..peak_index).rev());
    1e3 * (right - left)
}

pub fn analyse(spec: &Spec) -> Result<Analysis, String> {
    let lambda0 = spec.wavelength;
    let idx = indices(spec, lambda0)?;
    let period = match spec.period {
        PeriodSpec::Period(p) => p,
        PeriodSpec::Angle(a) => grating::period_for_angle(idx.grating, lambda0, idx.clad, a),
    };
    if !(period > 0.0) {
        return Err("the requested angle cannot be reached: the period would be negative".into());
    }
    let angle = grating::angle(idx.grating, lambda0, period, idx.clad, 1).ok_or("the first order does not radiate into the cladding at the design wavelength")?;
    let fibre = Fibre { mfd: spec.fibre_mfd, angle: spec.fibre_angle.unwrap_or(angle) };
    let apodisation = match spec.apodisation {
        ApodisationSpec::Uniform => Apodisation::Uniform,
        ApodisationSpec::Linear { from, to } => Apodisation::Linear { from, to },
        ApodisationSpec::Gaussian => Apodisation::Gaussian { radius: fibre.footprint(), centre: 0.5 * spec.length },
    };
    let emission = emit(spec.length, spec.leak, &apodisation, spec.samples);

    let up_at = |lambda: f64, sin_out: f64, n_clad: f64| -> f64 {
        if spec.substrate {
            upward_fraction(spec.up, n_clad, sin_out, spec.stack.n_box.index(lambda), spec.stack.box_thickness, spec.stack.n_sub.index(lambda), lambda)
        } else {
            spec.up
        }
    };
    let sin0 = grating::sin_angle(idx.grating, lambda0, period, idx.clad, 1);
    let (position, coupled) = match spec.fibre_position {
        Some(p) => (p, coupled_fraction(&emission, &fibre, p, sin0, idx.clad, lambda0)),
        None => best_position(&emission, &fibre, sin0, idx.clad, lambda0),
    };
    let up = up_at(lambda0, sin0, idx.clad);

    // the same grating and the same fibre at other wavelengths
    let efficiency_at = |lambda: f64| -> f64 {
        let Ok(i) = indices(spec, lambda) else { return 0.0 };
        let s = grating::sin_angle(i.grating, lambda, period, i.clad, 1);
        if s.abs() > 1.0 {
            return 0.0;
        }
        up_at(lambda, s, i.clad) * coupled_fraction(&emission, &fibre, position, s, i.clad, lambda)
    };
    let (from, to, points) = spec.sweep;
    let spectrum: Vec<(f64, f64)> = (0..points).map(|k| from + (to - from) * k as f64 / (points - 1) as f64).map(|l| (l, efficiency_at(l))).collect();
    let peak_index = (0..points).max_by(|a, b| spectrum[*a].1.partial_cmp(&spectrum[*b].1).unwrap()).unwrap();
    // refine the peak with a parabola through its neighbours
    let (mut peak_wavelength, mut peak_efficiency) = spectrum[peak_index];
    if peak_index > 0 && peak_index + 1 < points {
        let (a, b, c) = (spectrum[peak_index - 1].1, spectrum[peak_index].1, spectrum[peak_index + 1].1);
        let denom = a - 2.0 * b + c;
        if denom < 0.0 {
            let shift = 0.5 * (a - c) / denom;
            peak_wavelength += shift * (to - from) / (points - 1) as f64;
            peak_efficiency = b - 0.25 * (a - c) * shift;
        }
    }

    let h = 1e-4;
    let grating_index = |l: f64| indices(spec, l).map(|i| i.grating).unwrap_or(idx.grating);
    let group_index = idx.grating - lambda0 * (grating_index(lambda0 + h) - grating_index(lambda0 - h)) / (2.0 * h);
    let angle_at = |l: f64| indices(spec, l).ok().and_then(|i| grating::angle(i.grating, l, period, i.clad, 1)).unwrap_or(angle);
    let dtheta_dlambda = (angle_at(lambda0 + h) - angle_at(lambda0 - h)) / (2.0 * h) * 1e-3;
    let mismatch_rate = (group_index - idx.clad * fibre.angle.to_radians().sin()).abs();
    let bandwidth_1db_formula = 1e3 * 2.0 * (0.4 * 10f64.ln()).sqrt() * lambda0 * lambda0 / (2.0 * std::f64::consts::PI * fibre.footprint() * mismatch_rate);

    Ok(Analysis {
        indices: idx,
        group_index,
        period,
        angle,
        orders: grating::radiating_orders(idx.grating, lambda0, period, idx.clad),
        second_order_wavelength: grating::second_order_wavelength(idx.grating, period),
        overlap: if emission.radiated > 0.0 { coupled / emission.radiated } else { 0.0 },
        efficiency: up * coupled,
        emission,
        up,
        fibre,
        position,
        bandwidth_1db: width_below_peak(&spectrum, peak_index, 1.0),
        bandwidth_3db: width_below_peak(&spectrum, peak_index, 3.0102999566),
        spectrum,
        peak_wavelength,
        peak_efficiency,
        dtheta_dlambda,
        bandwidth_1db_formula,
    })
}
