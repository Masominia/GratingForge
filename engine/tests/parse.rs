use gratingforge::design::parse::parse;
use gratingforge::design::spec::{ApodisationSpec, PeriodSpec};
use gratingforge::optics::slab::Polarisation;

#[test]
fn defaults_describe_a_standard_coupler() {
    let s = parse("").unwrap();
    assert_eq!(s.wavelength, 1.55);
    assert_eq!(s.stack.core, 0.22);
    assert_eq!(s.period, PeriodSpec::Angle(10.0));
    assert_eq!(s.apodisation, ApodisationSpec::Uniform);
    assert!(!s.substrate);
}

#[test]
fn a_full_design() {
    let text = "# note\ntitle My coupler\nwavelength 1.31\nstack core=0.3 box=3\nmaterials core=3.5 clad=air\npolarisation tm\n\
                grating period=0.5 fill=0.4 etch=0.1 length=12\nleak 0.2\napodise linear from=0.1 to=0.3\nup 0.6\nsubstrate on\n\
                fibre mfd=9.2 angle=8 position=4\nsweep 1.26 1.36 101\nexpect angle 8 0.1 # comment\n";
    let s = parse(text).unwrap();
    assert_eq!(s.title, "My coupler");
    assert_eq!(s.polarisation, Polarisation::Tm);
    assert_eq!(s.period, PeriodSpec::Period(0.5));
    assert_eq!((s.fill, s.etch, s.length, s.leak, s.up), (0.4, 0.1, 12.0, 0.2, 0.6));
    assert_eq!(s.apodisation, ApodisationSpec::Linear { from: 0.1, to: 0.3 });
    assert_eq!(s.stack.n_core.index(1.31), 3.5);
    assert_eq!(s.stack.n_clad.index(1.31), 1.0);
    assert_eq!((s.fibre_mfd, s.fibre_angle, s.fibre_position), (9.2, Some(8.0), Some(4.0)));
    assert_eq!(s.sweep, (1.26, 1.36, 101));
    assert!(s.substrate);
    assert_eq!(s.expectations.len(), 1);
    assert_eq!(s.expectations[0].args, vec!["8", "0.1"]);
    assert_eq!(s.expectations[0].line, 14);
}

#[test]
fn errors_name_the_line() {
    for (text, needle) in [
        ("wavelength 1.55\nfrobnicate 3", "line 2: unknown statement"),
        ("wavelength -1", "line 1: wavelength must be positive"),
        ("grating fill=1.2", "between 0 and 1"),
        ("grating depth=3", "does not have 'depth'"),
        ("materials core=kryptonite", "neither a known material"),
        ("apodise wavy", "unknown apodisation"),
        ("apodise linear from=0.1", "needs to="),
        ("sweep 1.6 1.5 100", "from < to"),
        ("fibre mfd", "expected key=value"),
        ("substrate maybe", "on or off"),
        ("leak", "needs an argument"),
        ("up 1.5", "at most 1"),
    ] {
        let err = parse(text).unwrap_err();
        assert!(err.contains(needle), "{text:?} gave {err:?}");
    }
}

#[test]
fn an_etch_deeper_than_the_core_is_rejected() {
    assert!(parse("stack core=0.22\ngrating etch=0.3").unwrap_err().contains("deeper"));
    assert!(parse("stack core=0.22\ngrating etch=0.22").is_ok());
}
