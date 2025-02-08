//! Parser for design files.
//!
//! One statement per line, `#` starts a comment:
//!
//! ```text
//! title <text>
//! wavelength <um>
//! stack core=<um> [box=<um>]
//! materials [core=<name|n>] [clad=..] [box=..] [substrate=..]      names: si, sio2, sin, air
//! polarisation te|tm
//! grating period=<um> | angle=<deg>   [fill=..] [etch=<um>] [length=<um>]
//! leak <per um>
//! apodise uniform | linear from=.. to=.. | gaussian
//! up <fraction>
//! substrate on|off
//! fibre mfd=<um> [angle=<deg>] [position=<um>]
//! sweep <from um> <to um> <points>
//! expect <kind> <args...>
//! ```

use super::spec::{ApodisationSpec, Expectation, PeriodSpec, Spec};
use crate::optics::material::by_name;
use crate::optics::slab::Polarisation;

fn number(text: &str) -> Result<f64, String> {
    text.parse::<f64>().map_err(|_| format!("'{text}' is not a number"))
}

fn pairs<'a>(words: &[&'a str]) -> Result<Vec<(&'a str, &'a str)>, String> {
    words.iter().map(|w| w.split_once('=').filter(|(k, v)| !k.is_empty() && !v.is_empty()).ok_or_else(|| format!("expected key=value, got '{w}'"))).collect()
}

fn positive(key: &str, text: &str) -> Result<f64, String> {
    let v = number(text)?;
    if v > 0.0 {
        Ok(v)
    } else {
        Err(format!("{key} must be positive"))
    }
}

fn statement(spec: &mut Spec, line: &str, line_number: usize) -> Result<(), String> {
    let w: Vec<&str> = line.split_whitespace().collect();
    let arg = |i: usize| w.get(i).copied().ok_or_else(|| format!("'{}' needs an argument", w[0]));
    match w[0] {
        "title" => spec.title = line[5..].trim().to_string(),
        "wavelength" => spec.wavelength = positive("wavelength", arg(1)?)?,
        "stack" => {
            for (k, v) in pairs(&w[1..])? {
                match k {
                    "core" => spec.stack.core = positive(k, v)?,
                    "box" => spec.stack.box_thickness = positive(k, v)?,
                    _ => return Err(format!("stack does not have '{k}'")),
                }
            }
        }
        "materials" => {
            for (k, v) in pairs(&w[1..])? {
                let m = by_name(v)?;
                match k {
                    "core" => spec.stack.n_core = m,
                    "clad" => spec.stack.n_clad = m,
                    "box" => spec.stack.n_box = m,
                    "substrate" => spec.stack.n_sub = m,
                    _ => return Err(format!("materials does not have '{k}'")),
                }
            }
        }
        "polarisation" => spec.polarisation = Polarisation::parse(arg(1)?).ok_or("polarisation must be te or tm")?,
        "grating" => {
            for (k, v) in pairs(&w[1..])? {
                match k {
                    "period" => spec.period = PeriodSpec::Period(positive(k, v)?),
                    "angle" => spec.period = PeriodSpec::Angle(number(v)?),
                    "fill" => {
                        spec.fill = number(v)?;
                        if !(spec.fill > 0.0 && spec.fill < 1.0) {
                            return Err("fill must be strictly between 0 and 1".into());
                        }
                    }
                    "etch" => spec.etch = positive(k, v)?,
                    "length" => spec.length = positive(k, v)?,
                    _ => return Err(format!("grating does not have '{k}'")),
                }
            }
        }
        "leak" => spec.leak = positive("leak", arg(1)?)?,
        "apodise" => {
            spec.apodisation = match arg(1)? {
                "uniform" => ApodisationSpec::Uniform,
                "gaussian" => ApodisationSpec::Gaussian,
                "linear" => {
                    let (mut from, mut to) = (None, None);
                    for (k, v) in pairs(&w[2..])? {
                        match k {
                            "from" => from = Some(number(v)?),
                            "to" => to = Some(number(v)?),
                            _ => return Err(format!("linear apodisation does not have '{k}'")),
                        }
                    }
                    ApodisationSpec::Linear { from: from.ok_or("linear needs from=...")?, to: to.ok_or("linear needs to=...")? }
                }
                other => return Err(format!("unknown apodisation '{other}'")),
            }
        }
        "up" => {
            spec.up = number(arg(1)?)?;
            if !(spec.up > 0.0 && spec.up <= 1.0) {
                return Err("up must be above 0 and at most 1".into());
            }
        }
        "substrate" => {
            spec.substrate = match arg(1)? {
                "on" => true,
                "off" => false,
                _ => return Err("substrate must be on or off".into()),
            }
        }
        "fibre" => {
            for (k, v) in pairs(&w[1..])? {
                match k {
                    "mfd" => spec.fibre_mfd = positive(k, v)?,
                    "angle" => spec.fibre_angle = Some(number(v)?),
                    "position" => spec.fibre_position = Some(number(v)?),
                    _ => return Err(format!("fibre does not have '{k}'")),
                }
            }
        }
        "sweep" => {
            if w.len() != 4 {
                return Err("sweep needs: from to points".into());
            }
            let (from, to, points) = (positive("from", w[1])?, positive("to", w[2])?, number(w[3])?);
            if !(to > from) || points < 3.0 {
                return Err("sweep needs from < to and at least three points".into());
            }
            spec.sweep = (from, to, points as usize);
        }
        "expect" => spec.expectations.push(Expectation { kind: arg(1)?.to_string(), args: w[2..].iter().map(|s| s.to_string()).collect(), line: line_number }),
        other => return Err(format!("unknown statement '{other}'")),
    }
    Ok(())
}

/// Parses a design. Errors carry the line number.
pub fn parse(text: &str) -> Result<Spec, String> {
    let mut spec = Spec::default();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        statement(&mut spec, line, i + 1).map_err(|e| format!("line {}: {e}", i + 1))?;
    }
    if spec.etch > spec.stack.core + 1e-12 {
        return Err("the etch is deeper than the core is thick".into());
    }
    Ok(spec)
}
