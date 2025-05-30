//! Small numerical tools.

/// Root of a continuous function on [lo, hi] by bisection. Returns None if the ends have the same sign.
pub fn bisect<F: Fn(f64) -> f64>(f: F, mut lo: f64, mut hi: f64) -> Option<f64> {
    let (mut flo, fhi) = (f(lo), f(hi));
    if flo == 0.0 {
        return Some(lo);
    }
    if fhi == 0.0 {
        return Some(hi);
    }
    if (flo > 0.0) == (fhi > 0.0) {
        return None;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid);
        if fm == 0.0 || hi - lo < 1e-15 * (1.0 + mid.abs()) {
            return Some(mid);
        }
        if (fm > 0.0) == (flo > 0.0) {
            lo = mid;
            flo = fm;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Maximum of a unimodal function on [lo, hi] by golden-section search. Returns (position, value).
pub fn golden_max<F: Fn(f64) -> f64>(f: F, mut lo: f64, mut hi: f64) -> (f64, f64) {
    let r = 0.618_033_988_749_894_9;
    let (mut x1, mut x2) = (hi - r * (hi - lo), lo + r * (hi - lo));
    let (mut f1, mut f2) = (f(x1), f(x2));
    while hi - lo > 1e-10 * (1.0 + lo.abs() + hi.abs()) {
        if f1 > f2 {
            hi = x2;
            x2 = x1;
            f2 = f1;
            x1 = hi - r * (hi - lo);
            f1 = f(x1);
        } else {
            lo = x1;
            x1 = x2;
            f1 = f2;
            x2 = lo + r * (hi - lo);
            f2 = f(x2);
        }
    }
    let x = 0.5 * (lo + hi);
    (x, f(x))
}

/// Trapezoid rule on equally spaced samples.
pub fn trapezoid(values: &[f64], step: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    (values.iter().sum::<f64>() - 0.5 * (values[0] + values[values.len() - 1])) * step
}

/// Value at `x` of samples on a regular grid starting at `x0`, by linear interpolation; clamped at the ends.
pub fn interpolate(values: &[f64], x0: f64, step: f64, x: f64) -> f64 {
    let pos = ((x - x0) / step).clamp(0.0, (values.len() - 1) as f64);
    let i = (pos.floor() as usize).min(values.len() - 2);
    values[i] + (pos - i as f64) * (values[i + 1] - values[i])
}
