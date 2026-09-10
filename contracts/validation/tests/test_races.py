"""Mutate real inodes at observed stat/read boundaries without timing sleeps."""

from collections import Counter

import pytest
from support import boundary_cases, mutate_snapshot, observe_io, snapshot_files

from arboresce_contract_validation.cli import InputAdmissionError, SnapshotStore


@pytest.mark.parametrize("case", boundary_cases("snapshot-race"), ids=lambda row: row["id"])
def test_changed_file_never_returns_snapshot(case, tmp_path):
    root = tmp_path / "workspace"
    (path,) = snapshot_files(root, case["setup"])
    store = SnapshotStore(root / "contracts")
    with observe_io(store) as observed, mutate_snapshot(path, case, observed) as mutations:
        with pytest.raises(InputAdmissionError) as error:
            store.read(path)
        for text in case["expected"]["diagnostic_contains"]:
            assert text in str(error.value)
        assert mutations == [case["id"]]
        assert store.snapshots == {}
        if "requested_read_charge_max" in case["expected"]:
            assert store.requested_read_bytes <= case["expected"]["requested_read_charge_max"]
        if case["expected"].get("payload_reads") == 0:
            assert observed.reads == []
    assert Counter(observed.opened) == Counter(observed.closed)
