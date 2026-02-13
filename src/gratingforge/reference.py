"""An independent calculation of everything the Rust library reports, in NumPy and SciPy.

It shares no code with the library: its own design-file reader, a root find on the slab dispersion relation,
a closed form (complex error functions) for the overlap of a uniform grating with a Gaussian mode, and numerical
integration on a finer grid for apodised gratings.
"""

from __future__ import annotations

from pathlib import Path

import numpy as np
from scipy.optimize import brentq, minimize_scalar
from scipy.special import erf

MATERIALS = {
    "si": (3.476, -0.0865),
    "silicon": (3.476, -0.0865),
    "sio2": (1.444, -0.0120),
    "silica": (1.444, -0.0120),
    "oxide": (1.444, -0.0120),
    "sin": (1.996, -0.0240),
    "nitride": (1.996, -0.0240),
    "air": (1.0, 0.0),
}
REFERENCE_WAVELENGTH = 1.55
RADIATED_TARGET = 0.999

DEFAULTS = {
    "title": "",
    "wavelength": 1.55,
    "core": 0.22,
    "box": 2.0,
    "m_core": "si",
    "m_clad": "sio2",
    "m_box": "sio2",
    "m_substrate": "si",
    "polarisation": "te",
    "period": None,
    "angle": 10.0,
    "fill": 0.5,
    "etch": 0.07,
    "length": 15.0,
    "leak": 0.14,
    "apodise": ("uniform",),
    "up": 0.5,
    "substrate": False,
    "mfd": 10.4,
    "fibre_angle": None,
    "fibre_position": None,
    "sweep": (1.5, 1.6, 201),
}


def read_design(path: Path | str) -> dict:
    """Reads a design file into a flat dictionary, ignoring its expectations."""
    d = dict(DEFAULTS)
    for raw in Path(path).read_text().splitlines():
        w = raw.split("#")[0].split()
        if not w or w[0] == "expect":
            continue
        kv = dict(x.split("=") for x in w[1:] if "=" in x)
        if w[0] == "title":
            d["title"] = " ".join(w[1:])
        elif w[0] in ("wavelength", "leak", "up"):
            d[w[0]] = float(w[1])
        elif w[0] == "stack":
            d.update({k: float(v) for k, v in kv.items()})
        elif w[0] == "materials":
            d.update({f"m_{k}": v for k, v in kv.items()})
        elif w[0] == "polarisation":
            d["polarisation"] = w[1]
        elif w[0] == "grating":
            for k, v in kv.items():
                d[k] = float(v)
                if k == "period":
                    d["angle"] = None
                if k == "angle":
                    d["period"] = None
        elif w[0] == "apodise":
            d["apodise"] = ("linear", float(kv["from"]), float(kv["to"])) if w[1] == "linear" else (w[1],)
        elif w[0] == "substrate":
            d["substrate"] = w[1] == "on"
        elif w[0] == "fibre":
            names = {"mfd": "mfd", "angle": "fibre_angle", "position": "fibre_position"}
            d.update({names[k]: float(v) for k, v in kv.items()})
        elif w[0] == "sweep":
            d["sweep"] = (float(w[1]), float(w[2]), int(w[3]))
    return d


def index(material: str, lam: float) -> float:
    if material in MATERIALS:
        n0, slope = MATERIALS[material]
        return n0 + slope * (lam - REFERENCE_WAVELENGTH)
    return float(material)


def slab_index(lam: float, thickness: float, n_core: float, n_clad: float, n_sub: float, pol: str, order: int = 0):
    """Effective index of a three-layer slab mode, or None below cut-off."""
    k0 = 2 * np.pi / lam
    floor = max(n_clad, n_sub)
    c_clad, c_sub = ((n_core / n_clad) ** 2, (n_core / n_sub) ** 2) if pol == "tm" else (1.0, 1.0)

    def mismatch(n):
        kappa = k0 * np.sqrt(n_core**2 - n**2)
        gamma, delta = k0 * np.sqrt(max(n**2 - n_clad**2, 0.0)), k0 * np.sqrt(max(n**2 - n_sub**2, 0.0))
        return kappa * thickness - np.arctan2(c_clad * gamma, kappa) - np.arctan2(c_sub * delta, kappa) - order * np.pi

    lo, hi = floor + 1e-12, n_core - 1e-12
    if mismatch(lo) <= 0:
        return None
    return brentq(mismatch, lo, hi, xtol=1e-14)


def indices(d: dict, lam: float) -> dict:
    nc, ncl, nb = index(d["m_core"], lam), index(d["m_clad"], lam), index(d["m_box"], lam)
    full = slab_index(lam, d["core"], nc, ncl, nb, d["polarisation"])
    rest = d["core"] - d["etch"]
    etched = slab_index(lam, rest, nc, ncl, nb, d["polarisation"]) if rest > 1e-9 else None
    etched = max(ncl, nb) if etched is None else etched
    return {"full": full, "etched": etched, "grating": d["fill"] * full + (1 - d["fill"]) * etched, "clad": ncl}


def leakage_profile(d: dict, x: np.ndarray, footprint: float) -> np.ndarray:
    kind = d["apodise"][0]
    if kind == "uniform":
        return np.full_like(x, d["leak"])
    if kind == "linear":
        return np.maximum(d["apodise"][1] + (d["apodise"][2] - d["apodise"][1]) * x / d["length"], 0.0)
    g2 = np.exp(-2 * (x - 0.5 * d["length"]) ** 2 / footprint**2)
    step = x[1] - x[0]
    cumulative = np.concatenate([[0.0], np.cumsum(0.5 * (g2[1:] + g2[:-1]) * step)])
    share = RADIATED_TARGET * cumulative / cumulative[-1]
    return np.minimum(RADIATED_TARGET * g2 / cumulative[-1] / np.maximum(1 - share, 1e-9), d["leak"])


class Emitted:
    """Emitted field on a grid, with the coupled fraction to a Gaussian mode."""

    def __init__(self, d: dict, footprint: float, samples: int = 12001):
        self.d, self.w = d, footprint
        self.uniform = d["apodise"][0] == "uniform"
        self.x = np.linspace(0.0, d["length"], samples)
        self.step = self.x[1] - self.x[0]
        self.leakage = leakage_profile(d, self.x, footprint)
        exponent = np.concatenate([[0.0], np.cumsum(0.5 * (self.leakage[1:] + self.leakage[:-1]) * self.step)])
        self.amplitude = np.sqrt(self.leakage * np.exp(-exponent))
        self.radiated = 1 - np.exp(-d["leak"] * d["length"]) if self.uniform else 1 - np.exp(-exponent[-1])

    def coupled(self, position: float, dk: float) -> float:
        w, norm = self.w, self.w * np.sqrt(np.pi / 2)
        if self.uniform:
            # integral of sqrt(L) exp(-a x) exp(-(x - p)^2 / w^2) over the grating, a = L / 2 - i dk
            a = 0.5 * self.d["leak"] - 1j * dk
            span = erf((self.d["length"] - position) / w + 0.5 * a * w) - erf(-position / w + 0.5 * a * w)
            integral = (
                np.sqrt(self.d["leak"]) * np.exp(-a * position + 0.25 * a * a * w * w) * 0.5 * w * np.sqrt(np.pi) * span
            )
            return float(abs(integral) ** 2 / norm)
        f = self.amplitude * np.exp(-((self.x - position) ** 2) / w**2) * np.exp(1j * dk * self.x)
        integral = (f.sum() - 0.5 * (f[0] + f[-1])) * self.step
        return float(abs(integral) ** 2 / norm)


def upward(d: dict, lam: float, sin_out: float, n_clad: float) -> float:
    if not d["substrate"]:
        return d["up"]
    n_box, n_sub = index(d["m_box"], lam), index(d["m_substrate"], lam)
    s = n_clad * sin_out / n_box
    if abs(s) >= 1:
        return d["up"]
    c1 = np.sqrt(1 - s * s)
    s2 = n_box * s / n_sub
    r = (
        -1.0
        if abs(s2) >= 1
        else (n_box * c1 - n_sub * np.sqrt(1 - s2 * s2)) / (n_box * c1 + n_sub * np.sqrt(1 - s2 * s2))
    )
    phase = 4 * np.pi / lam * n_box * d["box"] * c1
    total = abs(np.sqrt(d["up"]) + np.sqrt(1 - d["up"]) * r * np.exp(1j * phase)) ** 2
    return float(total / (total + (1 - d["up"]) * (1 - r * r)))


def _width(lam: np.ndarray, eff: np.ndarray, drop_db: float) -> float:
    peak = int(np.argmax(eff))
    level = eff[peak] * 10 ** (-drop_db / 10)
    edges = []
    for order in (range(peak + 1, len(lam)), range(peak - 1, -1, -1)):
        previous, edge = peak, np.nan
        for i in order:
            if eff[i] <= level:
                edge = lam[previous] + (eff[previous] - level) / (eff[previous] - eff[i]) * (lam[i] - lam[previous])
                break
            previous = i
        edges.append(edge)
    return 1e3 * (edges[0] - edges[1])


def evaluate(d: dict, spectrum_points: int = 801) -> dict:
    """Every quantity the library reports for a design, computed here from scratch."""
    lam0 = d["wavelength"]
    idx = indices(d, lam0)
    period = (
        d["period"]
        if d["period"] is not None
        else lam0 / (idx["grating"] - idx["clad"] * np.sin(np.radians(d["angle"])))
    )
    sin0 = (idx["grating"] - lam0 / period) / idx["clad"]
    angle = float(np.degrees(np.arcsin(sin0)))
    fibre_angle = angle if d["fibre_angle"] is None else d["fibre_angle"]
    footprint = 0.5 * d["mfd"] / np.cos(np.radians(fibre_angle))
    emitted = Emitted(d, footprint)
    sin_fibre = np.sin(np.radians(fibre_angle))

    def dk(lam, sin_out, n_clad):
        return 2 * np.pi / lam * n_clad * (sin_out - sin_fibre)

    if d["fibre_position"] is None:
        scan = np.linspace(-0.25 * d["length"], 1.25 * d["length"], 301)
        values = [emitted.coupled(p, dk(lam0, sin0, idx["clad"])) for p in scan]
        best = scan[int(np.argmax(values))]
        fit = minimize_scalar(
            lambda p: -emitted.coupled(p, dk(lam0, sin0, idx["clad"])),
            bounds=(best - 0.2, best + 0.2),
            method="bounded",
            options={"xatol": 1e-9},
        )
        position = float(fit.x)
    else:
        position = d["fibre_position"]
    coupled = emitted.coupled(position, dk(lam0, sin0, idx["clad"]))
    up = upward(d, lam0, sin0, idx["clad"])

    def efficiency(lam):
        i = indices(d, lam)
        if i["full"] is None:
            return 0.0
        s = (i["grating"] - lam / period) / i["clad"]
        if abs(s) > 1:
            return 0.0
        return upward(d, lam, s, i["clad"]) * emitted.coupled(position, dk(lam, s, i["clad"]))

    lam = np.linspace(d["sweep"][0], d["sweep"][1], spectrum_points)
    eff = np.array([efficiency(v) for v in lam])
    k = int(np.argmax(eff))
    if 0 < k < len(lam) - 1:
        fit = minimize_scalar(
            lambda v: -efficiency(v), bounds=(lam[k - 1], lam[k + 1]), method="bounded", options={"xatol": 1e-10}
        )
        peak_wavelength, peak_efficiency = float(fit.x), float(-fit.fun)
    else:
        peak_wavelength, peak_efficiency = float(lam[k]), float(eff[k])

    h = 2e-5
    slope = (indices(d, lam0 + h)["grating"] - indices(d, lam0 - h)["grating"]) / (2 * h)
    group = idx["grating"] - lam0 * slope

    def angle_at(v):
        i = indices(d, v)
        return np.degrees(np.arcsin((i["grating"] - v / period) / i["clad"]))

    orders = sum(abs((idx["grating"] - m * lam0 / period) / idx["clad"]) <= 1 for m in range(1, 21))
    return {
        "n_full": idx["full"],
        "n_etched": idx["etched"],
        "n_grating": idx["grating"],
        "group_index": float(group),
        "period": float(period),
        "angle": angle,
        "orders": int(orders),
        "second_order_wavelength": float(idx["grating"] * period),
        "dtheta_dlambda": float((angle_at(lam0 + h) - angle_at(lam0 - h)) / (2 * h) * 1e-3),
        "radiated": float(emitted.radiated),
        "up": up,
        "overlap": coupled / float(emitted.radiated),
        "efficiency": up * coupled,
        "fibre_position": position,
        "peak_wavelength": peak_wavelength,
        "peak_efficiency": peak_efficiency,
        "bandwidth_1db": _width(lam, eff, 1.0),
        "bandwidth_3db": _width(lam, eff, 3.0102999566),
        "bandwidth_1db_formula": float(
            1e3
            * 2
            * np.sqrt(0.4 * np.log(10))
            * lam0**2
            / (2 * np.pi * footprint * abs(group - idx["clad"] * sin_fibre))
        ),
        "spectrum": (lam, eff),
    }


def uniform_overlap_limit() -> tuple[float, float]:
    """Largest overlap of a decaying exponential with a Gaussian, and the leakage (times the 1/e^2 radius) there."""

    def negative(product):
        d = dict(DEFAULTS, leak=product, length=60.0)
        e = Emitted(d, 1.0)
        return -max(e.coupled(p, 0.0) for p in np.linspace(0.0, 3.0, 601))

    fit = minimize_scalar(negative, bounds=(0.5, 3.0), method="bounded", options={"xatol": 1e-6})
    return float(-fit.fun), float(fit.x)
