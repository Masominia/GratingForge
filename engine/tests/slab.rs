use gratingforge::optics::slab::{effective_index, mode_count, Polarisation};

const L: f64 = 1.55;

#[test]
fn index_lies_between_cladding_and_core() {
    for t in [0.1, 0.22, 0.5] {
        let n = effective_index(L, t, 3.476, 1.444, 1.444, Polarisation::Te, 0).unwrap();
        assert!(n > 1.444 && n < 3.476);
    }
}

#[test]
fn standard_soi_slab() {
    // 220 nm silicon in oxide: the textbook value is close to 2.85
    let n = effective_index(L, 0.22, 3.476, 1.444, 1.444, Polarisation::Te, 0).unwrap();
    assert!((n - 2.848).abs() < 2e-3, "{n}");
}

#[test]
fn the_root_satisfies_the_symmetric_slab_equation() {
    // even TE modes of a symmetric slab: tan(kappa t / 2) = gamma / kappa
    let (nc, ncl, t) = (3.476, 1.444, 0.22);
    let n = effective_index(L, t, nc, ncl, ncl, Polarisation::Te, 0).unwrap();
    let k = 2.0 * std::f64::consts::PI / L;
    let (kappa, gamma) = (k * (nc * nc - n * n).sqrt(), k * (n * n - ncl * ncl).sqrt());
    assert!(((kappa * t / 2.0).tan() - gamma / kappa).abs() < 1e-8);
}

#[test]
fn thicker_slabs_have_higher_index() {
    let n = |t: f64| effective_index(L, t, 3.476, 1.444, 1.444, Polarisation::Te, 0).unwrap();
    assert!(n(0.15) < n(0.22) && n(0.22) < n(0.34));
}

#[test]
fn tm_is_below_te_and_higher_orders_below_the_fundamental() {
    let te = effective_index(L, 0.34, 3.476, 1.444, 1.444, Polarisation::Te, 0).unwrap();
    let tm = effective_index(L, 0.34, 3.476, 1.444, 1.444, Polarisation::Tm, 0).unwrap();
    let te1 = effective_index(L, 0.34, 3.476, 1.444, 1.444, Polarisation::Te, 1).unwrap();
    assert!(tm < te && te1 < te);
}

#[test]
fn mode_count_follows_the_cut_off_thickness() {
    // symmetric slab: mode m appears at t = m lambda / (2 sqrt(nc^2 - ncl^2))
    let step = L / (2.0 * (3.476f64.powi(2) - 1.444f64.powi(2)).sqrt());
    assert_eq!(mode_count(L, 0.9 * step, 3.476, 1.444, 1.444, Polarisation::Te), 1);
    assert_eq!(mode_count(L, 1.1 * step, 3.476, 1.444, 1.444, Polarisation::Te), 2);
    assert_eq!(mode_count(L, 2.1 * step, 3.476, 1.444, 1.444, Polarisation::Te), 3);
}

#[test]
fn asymmetric_slab_has_a_cut_off() {
    // air above, oxide below: a very thin film guides nothing
    assert!(effective_index(L, 0.01, 3.476, 1.0, 1.444, Polarisation::Te, 0).is_none());
    assert!(effective_index(L, 0.1, 3.476, 1.0, 1.444, Polarisation::Te, 0).is_some());
}

#[test]
fn no_mode_without_index_contrast() {
    assert!(effective_index(L, 0.22, 1.444, 1.444, 1.444, Polarisation::Te, 0).is_none());
    assert!(effective_index(L, 0.0, 3.476, 1.444, 1.444, Polarisation::Te, 0).is_none());
}
