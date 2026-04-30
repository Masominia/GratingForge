"""Recompute one design with the independent SciPy calculation and print both columns.

python scripts/compare_reference.py cases/mat_reference.gc
"""

import sys

from gratingforge import engine
from gratingforge.compare import TOLERANCES, compare

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    result = engine.run(sys.argv[1])
    cmp = compare(sys.argv[1], result)
    print(f"{'quantity':<26}{'library':>16}{'difference':>14}")
    for name in TOLERANCES:
        if name in cmp["differences"]:
            print(f"{name:<26}{result['metrics'][name]:>16.8f}{cmp['differences'][name]:>14.1e}")
    print(f"largest spectrum difference {cmp['spectrum']:.1e}")
    sys.exit(1 if cmp["outside"] else 0)
