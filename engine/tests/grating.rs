use gratingforge::optics::grating::{angle, mean_index, period_for_angle, radiating_orders, second_order_wavelength, sin_angle};

#[test]
fn mean_index_is_the_fill_weighted_average() {
    assert!((mean_index(2.8, 2.4, 0.5) - 2.6).abs() < 1e-12);
    assert!((mean_index(2.8, 2.4, 0.75) - 2.7).abs() < 1e-12);
}

#[test]
fn period_and_angle_are_inverse() {
    for a in [-15.0, 0.0, 8.0, 14.5, 30.0] {
        let p = period_for_angle(2.7, 1.55, 1.444, a);
        assert!((angle(2.7, 1.55, p, 1.444, 1).unwrap() - a).abs() < 1e-10);
    }
}

#[test]
fn vertical_emission_at_the_second_order_wavelength() {
    let p = period_for_angle(2.7, 1.55, 1.444, 0.0);
    assert!((second_order_wavelength(2.7, p) - 1.55).abs() < 1e-12);
    assert!(sin_angle(2.7, 1.55, p, 1.444, 1).abs() < 1e-12);
}

#[test]
fn longer_wavelength_tilts_the_beam_backward() {
    let p = 0.63;
    assert!(angle(2.7, 1.60, p, 1.444, 1).unwrap() < angle(2.7, 1.50, p, 1.444, 1).unwrap());
}

#[test]
fn evanescent_orders_are_reported() {
    // a period far too short: the first order stays guided
    assert!(angle(2.7, 1.55, 0.3, 1.444, 1).is_none());
    assert!(radiating_orders(2.7, 1.55, 0.3, 1.444).is_empty());
    assert_eq!(radiating_orders(2.7, 1.55, 0.63, 1.444), vec![1]);
    // orders 1, 2 and 3 have |sin| of 0.98, 0.08 and 0.81
    assert_eq!(radiating_orders(2.7, 1.55, 1.2, 1.444), vec![1, 2, 3]);
}
