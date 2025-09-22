import numpy as np
import pytest

from gratingforge import engine, reference
from gratingforge.compare import compare
from gratingforge.engine import ROOT


def test_symmetric_slab_satisfies_the_even_mode_equation():
    n = reference.slab_index(1.55, 0.22, 3.476, 1.444, 1.444, "te")
    k = 2 * np.pi / 1.55
    kappa, gamma = k * np.sqrt(3.476**2 - n**2), k * np.sqrt(n**2 - 1.444**2)
    assert abs(np.tan(kappa * 0.11) - gamma / kappa) < 1e-9


def test_slab_matches_a_finite_difference_eigenmode():
    # a third route to the same number: the TE wave equation on a grid
    n_core, n_clad, t, lam = 3.476, 1.444, 0.22, 1.55
    x = np.linspace(-1.5, 1.5, 6001)
    h = x[1] - x[0]
    eps = np.where(np.abs(x) < t / 2 - 1e-9, n_core**2, n_clad**2)
    eps[np.abs(np.abs(x) - t / 2) < 1e-9] = 0.5 * (n_core**2 + n_clad**2)  # nodes on the interfaces
    k = 2 * np.pi / lam
    main, off = -2 / h**2 + k**2 * eps, np.full(len(x) - 1, 1 / h**2)
    from scipy.linalg import eigh_tridiagonal

    top = eigh_tridiagonal(main, off, select="i", select_range=(len(x) - 1, len(x) - 1))[0][0]
    assert abs(np.sqrt(top) / k - reference.slab_index(lam, t, n_core, n_clad, n_clad, "te")) < 2e-4


def test_cut_off_returns_none():
    assert reference.slab_index(1.55, 0.01, 3.476, 1.0, 1.444, "te") is None


def test_closed_form_overlap_matches_quadrature():
    d = dict(reference.DEFAULTS, leak=0.17, length=22.0)
    e = reference.Emitted(d, 5.3)
    for position, dk in ((4.0, 0.0), (6.5, 0.05), (2.0, -0.12)):
        f = e.amplitude * np.exp(-((e.x - position) ** 2) / 5.3**2) * np.exp(1j * dk * e.x)
        numeric = abs((f.sum() - 0.5 * (f[0] + f[-1])) * e.step) ** 2 / (5.3 * np.sqrt(np.pi / 2))
        assert abs(e.coupled(position, dk) - numeric) < 1e-8


def test_uniform_limit():
    limit, product = reference.uniform_overlap_limit()
    assert abs(limit - 0.8010) < 2e-4 and abs(product - 1.369) < 2e-3


def test_reader_understands_every_statement(tmp_path):
    path = tmp_path / "d.gc"
    path.write_text(
        "# note\ntitle A B\nwavelength 1.31\nstack core=0.3 box=3\nmaterials core=3.5 clad=air\npolarisation tm\n"
        "grating period=0.5 fill=0.4 etch=0.1 length=12\nleak 0.2\napodise linear from=0.1 to=0.3\nup 0.6\n"
        "substrate on\nfibre mfd=9.2 angle=8 position=4\nsweep 1.26 1.36 101\nexpect angle 8 0.1\n"
    )
    d = reference.read_design(path)
    assert (d["title"], d["wavelength"], d["core"], d["box"]) == ("A B", 1.31, 0.3, 3.0)
    assert (d["m_core"], d["m_clad"], d["polarisation"]) == ("3.5", "air", "tm")
    assert (d["period"], d["angle"], d["fill"], d["etch"], d["length"]) == (0.5, None, 0.4, 0.1, 12.0)
    assert d["apodise"] == ("linear", 0.1, 0.3) and d["substrate"] and d["up"] == 0.6
    assert (d["mfd"], d["fibre_angle"], d["fibre_position"], d["sweep"]) == (9.2, 8.0, 4.0, (1.26, 1.36, 101))


@pytest.mark.skipif(not engine.available(), reason="Rust library not built")
@pytest.mark.parametrize("name", ["uniform_leak120", "apod_gauss_cap040", "fibre_tilt_m2", "box_thick22", "mat_tm"])
def test_library_and_reference_agree(name):
    path = ROOT / "cases" / f"{name}.gc"
    cmp = compare(path, engine.run(path))
    assert not cmp["outside"], cmp["outside"]
    assert cmp["spectrum"] < 2e-5
