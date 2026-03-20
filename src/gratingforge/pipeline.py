"""Analyses the whole design library, checks it and writes the report."""

from __future__ import annotations

import json
from pathlib import Path

import pandas as pd
import yaml

from . import cases, engine, plotting, reference
from .compare import compare
from .engine import ROOT
from .report import md_table


def run(cfg_path=None, out=None, fast: bool = False) -> dict:
    cfg = yaml.safe_load(open(cfg_path or ROOT / "configs" / "default.yaml"))
    out = Path(out) if out else ROOT / "reports"
    (out / "figures" / "cases").mkdir(parents=True, exist_ok=True)
    if not engine.available():
        raise SystemExit("Rust library not built: run `make build` first")
    paths = cases.discover(ROOT / cfg["cases_dir"])
    if fast:
        paths = [p for i, p in enumerate(paths) if i % cfg["fast_every"] == 0 or p.stem in cfg["gallery"]]

    checks, rows, gallery, worst = [], [], {}, {}
    for path in paths:
        record, result = cases.run_case(path)
        family = cases.family_of(path.stem, cfg["families"])
        cmp = compare(path, result)
        record["against_reference"] = {"largest_differences": cmp["differences"], "spectrum": cmp["spectrum"]}
        for name, diff in cmp["differences"].items():
            worst[name] = max(worst.get(name, 0.0), diff)
        worst["spectrum"] = max(worst.get("spectrum", 0.0), cmp["spectrum"])
        checks.append(
            {
                "area": "independent calculation",
                "check": f"{path.stem}: every reported quantity and the spectrum recomputed in SciPy",
                "status": "FAIL" if cmp["outside"] else "PASS",
                "detail": "; ".join(cmp["outside"])
                or f"{len(cmp['differences'])} quantities agree, spectrum within {cmp['spectrum']:.1e}",
            }
        )
        cases.write_record(record, out / "cases")
        plotting.case_figure(
            result, record["title"], cmp["reference_spectrum"], out / "figures" / "cases" / f"{path.stem}.png"
        )
        for c in record["checks"]:
            checks.append(
                {"area": family, "check": f"{path.stem}: {c['check']}", "status": c["status"], "detail": c["detail"]}
            )
        m = result["metrics"]
        rows.append(
            {
                "design": path.stem,
                "family": family,
                "period, nm": 1e3 * m["period"],
                "angle, deg": m["angle"],
                "overlap": m["overlap"],
                "efficiency, dB": m["efficiency_db"],
                "1 dB width, nm": m["bandwidth_1db"],
                "checks": len(record["checks"]),
                "passed": sum(c["status"] == "PASS" for c in record["checks"]),
            }
        )
        if path.stem in cfg["gallery"]:
            gallery[path.stem] = (result, record["title"])

    table = pd.DataFrame(rows)
    fam = table.groupby("family", sort=False).agg(
        designs=("design", "size"), checks=("checks", "sum"), passed=("passed", "sum")
    )
    if gallery:
        plotting.overview({k: gallery[k] for k in cfg["gallery"] if k in gallery}, out / "figures" / "overview.png")
    limit, product = reference.uniform_overlap_limit()
    checks.append(
        {
            "area": "independent calculation",
            "check": "largest overlap of a uniform grating with a Gaussian mode",
            "status": "PASS" if abs(limit - 0.801) < 5e-4 and abs(product - 1.369) < 5e-3 else "FAIL",
            "detail": f"{limit:.4f} at leakage x beam radius = {product:.3f} (textbook: 0.801 at 1.37)",
        }
    )

    val = pd.DataFrame(checks)
    counts = val["status"].value_counts().to_dict()
    agreement = pd.DataFrame({"largest difference over the library": worst}).rename_axis("quantity")
    lines = [
        "# GratingForge report",
        "",
        f"{len(table)} designs, {len(val)} checks: {counts.get('PASS', 0)} PASS, {counts.get('FAIL', 0)} FAIL.",
        "",
        "Every design is a text file in `cases/`. Its expectations come from an independent calculation or a closed form, stated in the comments at the top of the file.",
        "Per-design results are in `reports/cases/`, emitted fields and spectra in `reports/figures/cases/`.",
        "",
        "## Families",
        "",
        md_table(fam, 0),
        "",
        "## Agreement with the independent calculation",
        "",
        "Each design is recomputed from its file by `reference.py` (SciPy root finding, a closed form for uniform gratings, a finer grid for apodised ones). Units as reported: indices, micrometres, degrees, fractions, nanometres for widths.",
        "",
        md_table(agreement, 10),
        "",
        "## All designs",
        "",
        md_table(table.drop(columns=["family"]), 3, index=False),
        "",
    ]
    (out / "report.md").write_text("\n".join(lines))
    (out / "validation.md").write_text("# Validation checks\n\n" + md_table(val, index=False) + "\n")
    metrics = {
        "designs": len(table),
        "families": fam.to_dict("index"),
        "largest_differences": worst,
        "uniform_overlap_limit": [limit, product],
        "designs_table": table.set_index("design").to_dict("index"),
        "checks": checks,
    }
    (out / "metrics.json").write_text(json.dumps(metrics, indent=1, default=float))
    return metrics
