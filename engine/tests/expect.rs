use gratingforge::design::analyse::analyse;
use gratingforge::design::expect::evaluate;
use gratingforge::design::parse::parse;

fn checks(text: &str) -> Vec<gratingforge::design::expect::Check> {
    let spec = parse(text).unwrap();
    evaluate(&spec, &analyse(&spec).unwrap())
}

#[test]
fn passing_and_failing_expectations() {
    let c = checks("grating angle=10\nexpect angle 10 1e-6\nexpect angle 11 1e-6\nexpect orders 1\nexpect orders 2");
    assert_eq!(c.iter().map(|x| x.pass).collect::<Vec<_>>(), vec![true, false, true, false]);
    assert_eq!(c[0].name, "angle 10 1e-6");
}

#[test]
fn every_kind_is_understood() {
    let kinds = ["n_full 2.85 0.1", "n_etched 2.54 0.1", "n_grating 2.69 0.1", "group_index 3.6 0.2", "period 0.63 0.05", "angle 10 0.1",
        "second_order 1.7 0.1", "dtheta_dlambda -0.087 0.1", "radiated 0.88 0.1", "up 0.5 1e-9", "overlap 0.75 0.1", "efficiency 0.35 0.1",
        "efficiency_db -4.5 1", "position 5 2", "peak_wavelength 1.55 0.01", "bandwidth_1db 36 0.2", "bandwidth_3db 63 0.2", "bandwidth_formula 0.3", "orders 1"];
    let text: String = kinds.iter().map(|k| format!("expect {k}\n")).collect();
    let c = checks(&text);
    assert_eq!(c.len(), kinds.len());
    for x in &c {
        assert!(x.pass, "{}: {}", x.name, x.detail);
    }
}

#[test]
fn malformed_expectations_fail_with_their_line() {
    let c = checks("leak 0.1\nexpect angle 10\nexpect wobble 1 2\nexpect period abc 1");
    assert!(c.iter().all(|x| !x.pass));
    assert!(c[0].detail.contains("line 2") && c[0].detail.contains("takes 2 arguments"));
    assert!(c[1].detail.contains("line 3") && c[1].detail.contains("unknown expectation"));
    assert!(c[2].detail.contains("line 4") && c[2].detail.contains("not a number"));
}
