"""Python side of the Rust library: runs the binary on a design file and parses its output."""

from __future__ import annotations

import subprocess
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / "engine" / "target" / "release" / "gratingforge"


def available() -> bool:
    return BINARY.exists()


def run(design: Path | str, command: str = "run") -> dict:
    """Analyse a design. Returns the reported quantities, the checks and, for `run`, the field and the spectrum.

    Raises RuntimeError with the library's message if the design is invalid.
    """
    proc = subprocess.run([str(BINARY), command, str(design)], capture_output=True, text=True)
    if proc.returncode == 2:
        raise RuntimeError(proc.stderr.strip())
    res: dict = {"metrics": {}, "checks": [], "title": ""}
    field, spectrum = [], []
    for line in proc.stdout.splitlines():
        key, _, rest = line.partition(" ")
        if key == "field":
            field.append(rest.split())
        elif key == "spectrum":
            spectrum.append(rest.split())
        elif key == "check":
            status, _, tail = rest.partition(" ")
            name, _, detail = tail.partition(" :: ")
            res["checks"].append({"check": name, "status": status, "detail": detail})
        elif key == "title":
            res["title"] = rest
        else:
            res["metrics"][key] = float(rest)
    if field:
        res["field"] = np.array(field, dtype=float)  # columns: position, amplitude, leakage
        res["spectrum"] = np.array(spectrum, dtype=float)  # columns: wavelength, efficiency
    return res
