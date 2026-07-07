import json

import pytest

from gratingforge import cases, engine
from gratingforge.engine import ROOT

ALL = cases.discover(ROOT / "cases")


def test_library_is_present_and_described():
    assert len(ALL) >= 90
    for path in ALL:
        assert len(cases.notes(path)) >= 2, f"{path.name} has no description"
        assert "expect " in path.read_text(), f"{path.name} checks nothing"


def test_families():
    fam = {"slab": ["slab_"], "bandwidth": ["bw_"]}
    assert cases.family_of("slab_tm220", fam) == "slab"
    assert cases.family_of("bw_sin", fam) == "bandwidth"
    assert cases.family_of("box_thick10", fam) == "other"


@pytest.mark.skipif(not engine.available(), reason="Rust library not built")
@pytest.mark.parametrize("path", ALL, ids=lambda p: p.stem)
def test_every_design_meets_its_expectations(path):
    result = engine.run(path, "check")
    assert result["checks"], "no expectations evaluated"
    failed = [c for c in result["checks"] if c["status"] != "PASS"]
    assert not failed, failed


@pytest.mark.skipif(not engine.available(), reason="Rust library not built")
def test_record_round_trip(tmp_path):
    record, result = cases.run_case(ROOT / "cases" / "uniform_optimum.gc")
    back = json.loads(cases.write_record(record, tmp_path).read_text())
    assert back["case"] == "uniform_optimum" and abs(back["metrics"]["overlap"] - 0.801) < 1e-3
    assert "field" not in back and result["spectrum"].shape[1] == 2
