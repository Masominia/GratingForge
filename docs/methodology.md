# Methodology

GratingForge is a design model for surface grating couplers. It answers the questions that come before a full electromagnetic simulation: which period gives which angle, how much of the emitted field a fibre can collect, how wide the band is, and how the buried oxide changes the result. It does not solve Maxwell's equations for the grating. Two numbers that only such a solver can give, the leakage rate and the upward fraction of the grating alone, are inputs.

## 1. Slab modes

The grating is described by two slab sections: the unetched core and the core thinned by the etch. Each is a three-layer slab (cladding, core, buried oxide) whose effective index is the root of

    kappa t = atan(w_c gamma_c / kappa) + atan(w_s gamma_s / kappa) + m pi

with kappa = k sqrt(n_core^2 - n^2), gamma = k sqrt(n^2 - n_layer^2), w = 1 for TE and (n_core / n_layer)^2 for TM. The root is bracketed between the larger outer index and the core index and found by bisection. A section etched through, or too thin to guide, is given the larger outer index.

Materials carry a linear dispersion around 1.55 um (silicon, oxide, nitride); a number in place of a name is a constant index. The linear model is good over the C band and poor far from it, which is why the O-band designs of the library give indices as numbers.

## 2. Phase matching

The mean index of the grating is the fill-weighted average of the two section indices. Diffraction order m leaves into the cladding at

    n_clad sin(theta) = n_grating - m lambda / period

The library solves this for the period when an angle is requested, lists every order that radiates into the cladding, and reports the wavelength at which emission is vertical and the second order reflects the guided mode (n_grating x period).

The group index of the grating, n - lambda dn/dlambda, is obtained by a central difference of the mean index over wavelength; it includes both material and waveguide dispersion and sets the bandwidth.

## 3. Emitted field

Guided power decays as dP/dx = -L(x) P, where L is the power leakage rate. The emitted intensity is L(x) P(x) and the field amplitude its square root.

- uniform: constant L, an exponential field;
- linear: L ramps between two values;
- gaussian: L(x) = F g(x) / (1 - F G(x)), the profile that emits a Gaussian intensity g with cumulative G, radiating F = 99.9% of the power, clipped at the largest leakage the etch can provide (the `leak` statement).

The emitted phase is taken as linear along the grating, as for a constant period. In a real apodised grating the period must be adjusted along with the fill to keep it so.

## 4. Fibre overlap

The fibre mode is a Gaussian of the given mode field diameter, tilted by the fibre angle; its footprint on the chip has 1/e^2 radius MFD / (2 cos(angle)). The coupled fraction is the squared overlap integral of the emitted field with the normalised mode, including the phase mismatch k n_clad (sin(theta_emission) - sin(theta_fibre)) x. The fibre position along the grating is optimised (coarse scan, then golden section) unless fixed.

    efficiency = upward fraction x coupled fraction,   coupled fraction = radiated fraction x overlap

For a uniform grating the overlap cannot exceed 80.1%, reached when leakage x beam radius = 1.369; the library reproduces this limit.

## 5. Spectrum and bandwidth

The same grating and fibre are evaluated over the sweep; only the wavelength, the indices and hence the emission angle change. The peak is refined by a parabola through three points, and the 1 dB and 3 dB widths come from linear interpolation of the crossings. If a crossing lies outside the sweep the width is reported as NaN.

For a Gaussian emitted field matched to the fibre the 1 dB width has a closed form,

    width = 2 sqrt(0.4 ln 10) lambda^2 / (2 pi w |n_group - n_clad sin(theta)|)

with w the footprint radius. The apodised designs of the library agree with it within 1%.

## 6. Substrate

With `substrate on`, the wave radiated downward crosses the buried oxide, reflects from the oxide-silicon interface (Fresnel coefficient at the refracted angle) and adds to the upward wave with the round-trip phase 4 pi n_box t cos(theta_box) / lambda. The upward fraction is the resulting upward power over itself plus what enters the substrate. This is a two-beam estimate: multiple reflections, the phase of the grating's own up and down waves and the top-cladding interface are ignored. It gives the right period in oxide thickness, lambda / (2 n cos(theta)), and the right size of the swing, not the absolute position of the maxima.

## 7. Validation

Three layers:

1. Rust unit tests on each module (dispersion relation satisfied, cut-off thicknesses, grating equation inverses, power conservation of the emitted field, the Gaussian tilt law, Fresnel limits).
2. The design library: every file carries expectations, computed when the file was written by an independent NumPy and SciPy calculation or a closed form, and evaluated by the library itself.
3. The report pipeline recomputes every design from its file with `reference.py`, which shares no code with the library: its own file reader, SciPy root finding, an error-function closed form for uniform gratings and a finer grid for apodised ones. All reported quantities and the whole spectrum are compared.

Layers 2 and 3 use the same independent calculation, so they catch errors in the library's numerics and its file handling, not errors in the physical model itself. The model's limits are listed in the README.
