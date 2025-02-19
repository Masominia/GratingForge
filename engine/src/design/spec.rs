//! What a design file says.

use crate::optics::material::{Material, SILICA, SILICON};
use crate::optics::slab::Polarisation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stack {
    /// Core thickness, micrometres.
    pub core: f64,
    pub n_core: Material,
    pub n_clad: Material,
    pub n_box: Material,
    /// Buried-oxide thickness, used only when substrate interference is switched on.
    pub box_thickness: f64,
    pub n_sub: Material,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PeriodSpec {
    Period(f64),
    /// Solve the period for this emission angle in the cladding, degrees.
    Angle(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApodisationSpec {
    Uniform,
    Linear { from: f64, to: f64 },
    /// Match the footprint of the fibre mode, centred on the grating.
    Gaussian,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expectation {
    pub kind: String,
    pub args: Vec<String>,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub title: String,
    pub wavelength: f64,
    pub stack: Stack,
    pub polarisation: Polarisation,
    pub period: PeriodSpec,
    /// Fraction of each period that is left unetched.
    pub fill: f64,
    /// Etch depth, micrometres; equal to the core thickness for a fully etched grating.
    pub etch: f64,
    pub length: f64,
    /// Power leakage rate per micrometre: constant for a uniform grating, the upper limit for a Gaussian one.
    pub leak: f64,
    pub apodisation: ApodisationSpec,
    /// Upward fraction of the radiated power for the grating alone.
    pub up: f64,
    /// Include the two-beam estimate of the reflection from the substrate.
    pub substrate: bool,
    pub fibre_mfd: f64,
    /// Fibre tilt; None means "aligned with the emission at the design wavelength".
    pub fibre_angle: Option<f64>,
    /// Fibre position along the grating; None means "wherever the coupling is largest".
    pub fibre_position: Option<f64>,
    pub sweep: (f64, f64, usize),
    /// Samples of the emitted field along the grating.
    pub samples: usize,
    pub expectations: Vec<Expectation>,
}

impl Default for Spec {
    fn default() -> Self {
        Spec {
            title: String::new(),
            wavelength: 1.55,
            stack: Stack { core: 0.22, n_core: SILICON, n_clad: SILICA, n_box: SILICA, box_thickness: 2.0, n_sub: SILICON },
            polarisation: Polarisation::Te,
            period: PeriodSpec::Angle(10.0),
            fill: 0.5,
            etch: 0.07,
            length: 15.0,
            leak: 0.14,
            apodisation: ApodisationSpec::Uniform,
            up: 0.5,
            substrate: false,
            fibre_mfd: 10.4,
            fibre_angle: None,
            fibre_position: None,
            sweep: (1.5, 1.6, 201),
            samples: 3001,
            expectations: Vec::new(),
        }
    }
}
