use gratingforge::design::analyse::{analyse, indices};
use gratingforge::design::parse::parse;

fn run(text: &str) -> gratingforge::design::analyse::Analysis {
    analyse(&parse(text).unwrap()).unwrap()
}

#[test]
fn the_period_reproduces_the_requested_angle() {
    for a in ["-10", "0", "8", "25"] {
        let r = run(&format!("grating angle={a}"));
        assert!((r.angle - a.parse::<f64>().unwrap()).abs() < 1e-9);
    }
}

#[test]
fn efficiency_is_the_product_of_its_parts() {
    let r = run("grating angle=10 length=20\nleak 0.14\nup 0.6");
    assert!((r.efficiency - 0.6 * r.emission.radiated * r.overlap).abs() < 1e-12);
    assert!(r.overlap < 0.801);
}

#[test]
fn the_spectrum_peaks_at_the_design_wavelength() {
    let r = run("grating angle=10 length=30\nleak 0.14");
    assert!((r.peak_wavelength - 1.55).abs() < 2e-4);
    assert!((r.peak_efficiency - r.efficiency).abs() < 1e-6);
    assert!(r.bandwidth_3db > r.bandwidth_1db && r.bandwidth_1db > 20.0);
}

#[test]
fn apodised_bandwidth_matches_the_formula() {
    let r = run("grating angle=10 length=45\nleak 1.5\napodise gaussian\nsweep 1.42 1.68 521");
    assert!((r.bandwidth_1db / r.bandwidth_1db_formula - 1.0).abs() < 0.01);
}

#[test]
fn a_narrow_sweep_gives_no_bandwidth() {
    let r = run("grating angle=10\nsweep 1.549 1.551 11");
    assert!(r.bandwidth_1db.is_nan());
}

#[test]
fn group_index_exceeds_the_phase_index() {
    let r = run("");
    assert!(r.group_index > r.indices.grating + 0.5);
    assert!(r.dtheta_dlambda < 0.0);
}

#[test]
fn fixed_fibre_settings_are_respected() {
    let r = run("grating angle=10 length=30\nfibre angle=12 position=7");
    assert_eq!((r.fibre.angle, r.position), (12.0, 7.0));
    assert!(r.peak_wavelength < 1.55);
}

#[test]
fn impossible_designs_are_refused() {
    let thin = parse("stack core=0.01\nmaterials clad=air\ngrating etch=0.005").unwrap();
    assert!(indices(&thin, 1.55).is_err());
    assert!(analyse(&thin).is_err());
    let short = parse("grating period=0.3").unwrap();
    assert!(analyse(&short).unwrap_err().contains("does not radiate"));
}

#[test]
fn full_etch_uses_the_cladding_index() {
    let s = parse("grating etch=0.22").unwrap();
    assert!((indices(&s, 1.55).unwrap().etched - 1.444).abs() < 1e-12);
}
