use gratingforge::optics::material::{by_name, AIR, NITRIDE, SILICA, SILICON};

#[test]
fn indices_at_the_reference_wavelength() {
    assert_eq!(SILICON.index(1.55), 3.476);
    assert_eq!(SILICA.index(1.55), 1.444);
    assert_eq!(NITRIDE.index(1.55), 1.996);
    assert_eq!(AIR.index(1.3), 1.0);
}

#[test]
fn normal_dispersion_and_group_index() {
    assert!(SILICON.index(1.5) > SILICON.index(1.6));
    let ng = SILICON.group_index(1.55);
    assert!((ng - (3.476 + 1.55 * 0.0865)).abs() < 1e-12);
    assert!(ng > 3.6 && ng < 3.62);
    assert!((SILICA.group_index(1.55) - 1.4626).abs() < 1e-4);
}

#[test]
fn names_numbers_and_rubbish() {
    assert_eq!(by_name("si").unwrap(), SILICON);
    assert_eq!(by_name("oxide").unwrap(), SILICA);
    let m = by_name("2.2").unwrap();
    assert_eq!(m.index(1.3), 2.2);
    assert_eq!(m.group_index(1.3), 2.2);
    assert!(by_name("unobtainium").is_err());
    assert!(by_name("0.5").is_err());
}
