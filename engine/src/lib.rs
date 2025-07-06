//! A design model for waveguide grating couplers.
//!
//! The chain is: slab modes give the effective indices of the etched and unetched sections; their average
//! and the period give the emission angle; a leakage rate along the grating gives the emitted field;
//! the overlap of that field with a tilted fibre mode gives the coupling efficiency; repeating at other
//! wavelengths gives the bandwidth. Lengths are in micrometres, angles in degrees at the interfaces.
//!
//! This is a model for design reasoning, not an electromagnetic solver: the leakage rate and the intrinsic
//! upward fraction are inputs.

pub mod design;
pub mod math;
pub mod optics;

pub use design::spec::Spec;
pub use math::complex::C64;
