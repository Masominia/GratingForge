import pytest

from gratingforge import engine

pytestmark = pytest.mark.skipif(not engine.available(), reason="Rust library not built")


def write(tmp_path, text):
    path = tmp_path / "design.gc"
    path.write_text(text)
    return path


def test_run_returns_field_and_spectrum(tmp_path):
    r = engine.run(write(tmp_path, "title Test\ngrating angle=10 length=20\nsweep 1.5 1.6 101\n"))
    assert r["title"] == "Test"
    assert r["spectrum"].shape == (101, 2) and r["field"].shape[1] == 3
    assert abs(r["metrics"]["angle"] - 10) < 1e-6
    assert abs(r["field"][-1, 0] - 20) < 0.1


def test_check_reports_pass_and_fail(tmp_path):
    r = engine.run(write(tmp_path, "grating angle=10\nexpect angle 10 1e-6\nexpect angle 12 1e-6\n"), "check")
    assert [c["status"] for c in r["checks"]] == ["PASS", "FAIL"]
    assert "field" not in r


def test_bad_design_raises_with_the_line(tmp_path):
    with pytest.raises(RuntimeError, match="line 2"):
        engine.run(write(tmp_path, "wavelength 1.55\nnonsense 1\n"))
    with pytest.raises(RuntimeError, match="does not radiate"):
        engine.run(write(tmp_path, "grating period=0.3\n"))
    with pytest.raises(RuntimeError, match="cannot read"):
        engine.run(tmp_path / "missing.gc")


def test_directional_grating_is_more_efficient(tmp_path):
    low = engine.run(write(tmp_path, "up 0.5\n"), "check")["metrics"]["efficiency"]
    high = engine.run(write(tmp_path, "up 0.8\n"), "check")["metrics"]["efficiency"]
    assert abs(high / low - 1.6) < 1e-6
