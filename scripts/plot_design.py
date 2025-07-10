"""Draw the emitted field and the spectrum of one design.

python scripts/plot_design.py cases/apod_gauss_cap080.gc field.png
"""

import sys

from gratingforge import engine, plotting

if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    result = engine.run(sys.argv[1])
    plotting.case_figure(result, result["title"], None, sys.argv[2])
    print(f"wrote {sys.argv[2]}")
