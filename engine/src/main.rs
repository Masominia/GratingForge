//! Command line front end.
//!
//!     gratingforge run <design>     metrics, checks, the emitted field and the spectrum
//!     gratingforge check <design>   metrics and checks; exit status 1 if a check fails
//!
//! Output lines: `key value`, `check <PASS|FAIL> <name> :: <detail>`, and for `run`
//! `field <x um> <amplitude> <leakage per um>` and `spectrum <wavelength um> <efficiency>`.

use std::env;
use std::fs;
use std::process;

use gratingforge::design::analyse::analyse;
use gratingforge::design::expect::evaluate;
use gratingforge::design::parse::parse;

fn fail(msg: &str) -> ! {
    eprintln!("gratingforge: {msg}");
    process::exit(2)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 || (args[0] != "run" && args[0] != "check") {
        fail("usage: gratingforge <run|check> <design file>");
    }
    let text = fs::read_to_string(&args[1]).unwrap_or_else(|e| fail(&format!("cannot read {}: {e}", args[1])));
    let spec = parse(&text).unwrap_or_else(|e| fail(&e));
    let a = analyse(&spec).unwrap_or_else(|e| fail(&e));
    let db = |x: f64| 10.0 * x.max(1e-30).log10();

    println!("title {}", spec.title);
    println!("wavelength {}\nn_full {:.10}\nn_etched {:.10}\nn_grating {:.10}\ngroup_index {:.8}", spec.wavelength, a.indices.full, a.indices.etched, a.indices.grating, a.group_index);
    println!("period {:.8}\nangle {:.8}\norders {}\nsecond_order_wavelength {:.8}\ndtheta_dlambda {:.8}", a.period, a.angle, a.orders.len(), a.second_order_wavelength, a.dtheta_dlambda);
    println!("radiated {:.8}\nup {:.8}\noverlap {:.8}\nefficiency {:.8}\nefficiency_db {:.6}", a.emission.radiated, a.up, a.overlap, a.efficiency, db(a.efficiency));
    println!("fibre_angle {:.8}\nfibre_position {:.6}\nfibre_footprint {:.8}", a.fibre.angle, a.position, a.fibre.footprint());
    println!("peak_wavelength {:.8}\npeak_efficiency {:.8}\nbandwidth_1db {:.6}\nbandwidth_3db {:.6}\nbandwidth_1db_formula {:.6}", a.peak_wavelength, a.peak_efficiency, a.bandwidth_1db, a.bandwidth_3db, a.bandwidth_1db_formula);
    let mut failed = 0;
    for c in evaluate(&spec, &a) {
        println!("check {} {} :: {}", if c.pass { "PASS" } else { "FAIL" }, c.name, c.detail);
        failed += usize::from(!c.pass);
    }
    if args[0] == "check" {
        process::exit(if failed == 0 { 0 } else { 1 });
    }
    let stride = (a.emission.len() / 600).max(1);
    for i in (0..a.emission.len()).step_by(stride) {
        println!("field {:.5} {:.8} {:.8}", a.emission.position(i), a.emission.amplitude[i], a.emission.leakage[i]);
    }
    for (l, e) in &a.spectrum {
        println!("spectrum {l:.8} {e:.10}");
    }
}
