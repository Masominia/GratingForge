"""Compares what the library reports with the independent calculation."""

from __future__ import annotations

from pathlib import Path

import numpy as np

from . import reference

# quantity: (absolute tolerance, relative tolerance); a value passes if either is met
TOLERANCES = {
    "n_full": (1e-8, 0.0),
    "n_etched": (1e-8, 0.0),
    "n_grating": (1e-8, 0.0),
    "group_index": (1e-5, 0.0),
    "period": (1e-8, 0.0),
    "angle": (1e-6, 0.0),
    "orders": (0.0, 0.0),
    "second_order_wavelength": (1e-8, 0.0),
    "dtheta_dlambda": (1e-6, 0.0),
    "radiated": (1e-6, 0.0),
    "up": (1e-7, 0.0),
    "overlap": (1e-5, 0.0),
    "efficiency": (1e-5, 0.0),
    "peak_wavelength": (5e-5, 0.0),
    "peak_efficiency": (1e-5, 0.0),
    "bandwidth_1db": (0.0, 1e-3),
    "bandwidth_3db": (0.0, 1e-3),
    "bandwidth_1db_formula": (0.0, 1e-6),
}


def compare(path: Path, result: dict) -> dict:
    """Recomputes one design. Returns the quantities out of tolerance and the largest spectrum difference."""
    design = reference.read_design(path)
    ref = reference.evaluate(design, spectrum_points=design["sweep"][2])
    got = result["metrics"]
    worst, outside = {}, []
    for name, (tol_abs, tol_rel) in TOLERANCES.items():
        a, b = got[name], ref[name]
        if a != a and b != b:  # both undefined: the sweep does not contain the band edge
            continue
        diff = abs(a - b)
        worst[name] = diff
        if not (diff <= tol_abs or diff <= tol_rel * abs(b)):
            outside.append(f"{name}: {a:.8g} against {b:.8g}")
    spectrum = float(np.max(np.abs(result["spectrum"][:, 1] - ref["spectrum"][1])))
    if spectrum > 2e-5:
        outside.append(f"spectrum differs by {spectrum:.2e}")
    return {"outside": outside, "differences": worst, "spectrum": spectrum, "reference_spectrum": ref["spectrum"]}
