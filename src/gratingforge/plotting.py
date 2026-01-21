"""Static figures for the report (matplotlib, PNG for the README)."""

from __future__ import annotations

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402

# Validated categorical order (fixed, never cycled) + recessive ink.
SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"]
SURFACE, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e4e3df"
NEUTRAL = "#8a8985"
CRITICAL = "#c4271a"

plt.rcParams.update(
    {
        "figure.facecolor": SURFACE,
        "axes.facecolor": SURFACE,
        "savefig.facecolor": SURFACE,
        "axes.edgecolor": GRID,
        "axes.labelcolor": INK2,
        "xtick.color": INK2,
        "ytick.color": INK2,
        "text.color": INK,
        "axes.grid": True,
        "grid.color": GRID,
        "grid.linewidth": 0.6,
        "axes.spines.top": False,
        "axes.spines.right": False,
        "font.size": 9,
        "axes.titlesize": 10,
        "axes.titleweight": "bold",
        "legend.frameon": False,
        "lines.linewidth": 1.6,
        "figure.dpi": 150,
        "axes.axisbelow": True,
    }
)


def _save(fig, path):
    fig.tight_layout()
    fig.savefig(path, bbox_inches="tight")
    plt.close(fig)


def _field(ax, result):
    x, amplitude = result["field"][:, 0], result["field"][:, 1]
    m = result["metrics"]
    mode = np.exp(-((x - m["fibre_position"]) ** 2) / m["fibre_footprint"] ** 2)
    scale = max(amplitude.max(), 1e-12)
    ax.plot(x, amplitude / scale, color=SERIES[0], label="emitted field")
    ax.plot(x, mode, color=SERIES[1], lw=1.2, ls="--", label="fibre mode")
    ax.set_xlim(x[0], x[-1])
    ax.set_ylim(0, 1.18)
    ax.set(xlabel="position along the grating, um", ylabel="amplitude, normalised")
    ax.legend(loc="upper right", ncol=2, fontsize=7.5)


def _spectrum(ax, result, reference=None):
    lam, eff = 1e3 * result["spectrum"][:, 0], result["spectrum"][:, 1]
    db = 10 * np.log10(np.maximum(eff, 1e-12))
    top = db.max()
    ax.plot(lam, db, color=SERIES[0], label="library")
    if reference is not None:
        step = max(len(reference[0]) // 24, 1)
        ref_db = 10 * np.log10(np.maximum(reference[1][::step], 1e-12))
        ax.plot(1e3 * reference[0][::step], ref_db, ls="none", marker="o", ms=3.2, color=INK2, label="independent")
        ax.legend(loc="upper right", fontsize=7.5)
    ax.set_xlim(lam[0], lam[-1])
    ax.set_ylim(top - 12, top + 2.5)
    ax.set(xlabel="wavelength, nm", ylabel="coupling efficiency, dB")


def case_figure(result, title, reference, path):
    fig, (left, right) = plt.subplots(1, 2, figsize=(8.4, 3.2))
    _field(left, result)
    _spectrum(right, result, reference)
    m = result["metrics"]
    fig.suptitle(f"{title}: {m['efficiency_db']:.2f} dB at {m['angle']:.1f} deg", fontsize=9, fontweight="bold")
    fig.tight_layout()
    fig.savefig(path, dpi=105)
    plt.close(fig)


def overview(gallery, path):
    names = list(gallery)
    fig, axes = plt.subplots(2, len(names), figsize=(3.3 * len(names), 5.6), squeeze=False)
    for k, name in enumerate(names):
        result, title = gallery[name]
        _field(axes[0, k], result)
        _spectrum(axes[1, k], result)
        axes[0, k].set_title(title if len(title) < 44 else title[:41] + "...", fontsize=8)
    _save(fig, path)
