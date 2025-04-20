//! Expectations of a design file, checked against its analysis.
//!
//! ```text
//! n_full | n_etched | n_grating | group_index <value> <tol>     effective indices at the design wavelength
//! period <um> <tol>              grating period
//! angle <deg> <tol>              emission angle in the cladding
//! orders <n>                     number of diffraction orders radiating into the cladding
//! second_order <um> <tol>        wavelength of vertical emission and back-reflection
//! dtheta_dlambda <deg/nm> <reltol>
//! radiated <fraction> <tol>      share of the guided power that leaves the grating
//! up <fraction> <tol>            upward share of the radiated power
//! overlap <fraction> <tol>       overlap of the emitted field with the fibre mode
//! efficiency <fraction> <tol>    guided power to fibre; efficiency_db <dB> <tol> likewise
//! position <um> <tol>            fibre position along the grating
//! peak_wavelength <um> <tol>
//! bandwidth_1db | bandwidth_3db <nm> <reltol>
//! bandwidth_formula <reltol>     the 1 dB width agrees with the Gaussian-beam formula
//! ```

use super::analyse::Analysis;
use super::spec::{Expectation, Spec};

#[derive(Clone, Debug, PartialEq)]
pub struct Check {
    pub name: String,
    pub pass: bool,
    pub detail: String,
}

fn numbers(e: &Expectation, n: usize) -> Result<Vec<f64>, String> {
    if e.args.len() != n {
        return Err(format!("'{}' takes {n} argument{}", e.kind, if n == 1 { "" } else { "s" }));
    }
    e.args.iter().map(|a| a.parse::<f64>().map_err(|_| format!("'{a}' is not a number"))).collect()
}

fn absolute(got: f64, e: &Expectation) -> Result<(bool, String), String> {
    let v = numbers(e, 2)?;
    Ok(((got - v[0]).abs() <= v[1], format!("{got:.8}, expected {:.8}, off by {:.2e}", v[0], got - v[0])))
}

fn relative(got: f64, e: &Expectation) -> Result<(bool, String), String> {
    let v = numbers(e, 2)?;
    Ok(((got / v[0] - 1.0).abs() <= v[1], format!("{got:.6}, expected {:.6}, relative error {:.2e}", v[0], got / v[0] - 1.0)))
}

fn one(a: &Analysis, e: &Expectation) -> Result<(bool, String), String> {
    match e.kind.as_str() {
        "n_full" => absolute(a.indices.full, e),
        "n_etched" => absolute(a.indices.etched, e),
        "n_grating" => absolute(a.indices.grating, e),
        "group_index" => absolute(a.group_index, e),
        "period" => absolute(a.period, e),
        "angle" => absolute(a.angle, e),
        "second_order" => absolute(a.second_order_wavelength, e),
        "radiated" => absolute(a.emission.radiated, e),
        "up" => absolute(a.up, e),
        "overlap" => absolute(a.overlap, e),
        "efficiency" => absolute(a.efficiency, e),
        "efficiency_db" => absolute(10.0 * a.efficiency.max(1e-30).log10(), e),
        "position" => absolute(a.position, e),
        "peak_wavelength" => absolute(a.peak_wavelength, e),
        "dtheta_dlambda" => relative(a.dtheta_dlambda, e),
        "bandwidth_1db" => relative(a.bandwidth_1db, e),
        "bandwidth_3db" => relative(a.bandwidth_3db, e),
        "bandwidth_formula" => {
            let v = numbers(e, 1)?;
            let ratio = a.bandwidth_1db / a.bandwidth_1db_formula;
            Ok(((ratio - 1.0).abs() <= v[0], format!("{:.3} nm computed, {:.3} nm from the formula, ratio {ratio:.4}", a.bandwidth_1db, a.bandwidth_1db_formula)))
        }
        "orders" => {
            let v = numbers(e, 1)?;
            Ok((a.orders.len() as f64 == v[0], format!("{} radiating order(s): {:?}", a.orders.len(), a.orders)))
        }
        other => Err(format!("unknown expectation '{other}'")),
    }
}

/// A malformed expectation becomes a failed check with its line number; it does not stop the others.
pub fn evaluate(spec: &Spec, analysis: &Analysis) -> Vec<Check> {
    spec.expectations
        .iter()
        .map(|e| {
            let name = std::iter::once(e.kind.clone()).chain(e.args.iter().cloned()).collect::<Vec<_>>().join(" ");
            match one(analysis, e) {
                Ok((pass, detail)) => Check { name, pass, detail },
                Err(msg) => Check { name, pass: false, detail: format!("line {}: {msg}", e.line) },
            }
        })
        .collect()
}
