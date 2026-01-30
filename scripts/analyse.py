"""Analyse one design file and print what the library found.

python scripts/analyse.py cases/mat_reference.gc
"""

import sys

from gratingforge import engine

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    r = engine.run(sys.argv[1], "check")
    m = r["metrics"]
    print(r["title"])
    print(
        f"  indices: slab {m['n_full']:.4f}, etched {m['n_etched']:.4f}, grating {m['n_grating']:.4f}, group {m['group_index']:.3f}"
    )
    print(
        f"  period {1e3 * m['period']:.1f} nm, emission at {m['angle']:.2f} deg, {int(m['orders'])} radiating order(s)"
    )
    print(f"  radiated {m['radiated']:.3f}, upward {m['up']:.3f}, overlap {m['overlap']:.3f}")
    print(
        f"  efficiency {m['efficiency']:.3f} ({m['efficiency_db']:.2f} dB), fibre {m['fibre_position']:.2f} um along the grating"
    )
    print(
        f"  peak at {1e3 * m['peak_wavelength']:.1f} nm, 1 dB width {m['bandwidth_1db']:.1f} nm, 3 dB width {m['bandwidth_3db']:.1f} nm"
    )
    for c in r["checks"]:
        print(f"  {c['status']}  {c['check']}  ({c['detail']})")
