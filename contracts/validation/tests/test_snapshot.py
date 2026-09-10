"""Fixed filesystem admission and stable-byte regressions."""

from __future__ import annotations

import os
from collections import Counter

import pytest
from support import (
    boundary_cases,
    fixture_document,
    observe_audit,
    observe_io,
    snapshot_files,
    write_file,
)

from arboresce_contract_validation.cli import InputAdmissionError, SnapshotStore, SnapshotStream


@pytest.mark.parametrize(
    "case",
    [
        row
        for row in boundary_cases("snapshot")
        if "kind" not in row["setup"]
        and "root_kind" not in row["setup"]
        and "capability_absence_each" not in row["setup"]
    ],
    ids=lambda row: row["id"],
)
def test_snapshot_bytes_and_limits(case, tmp_path):
    root = tmp_path / "workspace"
    paths = snapshot_files(root, case["setup"])
    expected = case["expected"]
    store = SnapshotStore(root / "contracts")
    with observe_io(store) as observed:
        if expected.get("classification") == "admission":
            with pytest.raises(InputAdmissionError) as error:
                for path in paths:
                    before_last_reads = list(observed.reads)
                    store.read(path)
            for text in expected["diagnostic_contains"]:
                assert text in str(error.value)
            assert paths[-1] not in store.snapshots
            if expected.get("last_file_read_syscalls") == 0:
                assert observed.reads == before_last_reads
            if "requested_read_charge_max" in expected:
                assert store.requested_read_bytes <= expected["requested_read_charge_max"]
                assert sum(size for _, size, _ in observed.reads) == store.requested_read_bytes
                assert (
                    sum(size for _, size, _ in observed.reads)
                    <= expected["requested_read_charge_max"]
                )
        else:
            snapshots = [store.read(path) for path in paths]
            if "payload_bytes" in expected:
                assert sum(len(item.data) for item in snapshots) == expected["payload_bytes"]
            if "payload_utf8" in expected:
                assert snapshots[0].data == expected["payload_utf8"].encode()
            if "all_payload_bytes_ascii" in expected:
                assert snapshots[0].data.count(b"x") == expected["payload_bytes"]
            if case["id"] == "snapshot-repeat-identity":
                before = list(observed.reads)
                assert store.read(paths[0]) is snapshots[0]
                assert snapshots[0].data == b"abc"
                assert observed.reads == before
            assert sum(returned == 0 for _, _, returned in observed.reads) == len(paths)
        if "requested_read_charge" in expected:
            assert store.requested_read_bytes == expected["requested_read_charge"]
            assert sum(size for _, size, _ in observed.reads) == expected["requested_read_charge"]
        if "unique_files" in expected:
            assert len(store.snapshots) == expected["unique_files"]
        if expected.get("read_syscalls") == 0:
            assert observed.reads == []
    assert Counter(observed.opened) == Counter(observed.closed)
    assert store.root_fd is None


@pytest.mark.parametrize(
    "case",
    [row for row in boundary_cases("snapshot") if "kind" in row["setup"]],
    ids=lambda row: row["id"],
)
def test_special_files_and_root_escape(case, tmp_path, monkeypatch):
    root = tmp_path / "workspace"
    contracts = root / "contracts"
    contracts.mkdir(parents=True)
    setup, expected = case["setup"], case["expected"]
    path = root / setup["path"]
    kind = setup["kind"]
    if kind == "directory":
        path.mkdir()
    elif kind == "fifo-with-no-writer":
        os.mkfifo(path)
    elif kind == "symlink":
        write_file(path.parent / setup["target"], {"utf8": setup["target_utf8"]})
        path.symlink_to(setup["target"])
    elif kind == "parent-symlink":
        target = root / setup["target_parent"]
        write_file(target / path.name, {"utf8": setup["target_utf8"]})
        path.parent.symlink_to(target, target_is_directory=True)
    else:
        assert kind == "regular-outside-invocation-root"
        write_file(path, setup)
    monkeypatch.chdir(root)
    store = SnapshotStore(contracts)
    with observe_io(store) as observed:
        with pytest.raises(InputAdmissionError) as error:
            store.read(setup["path"])
        for text in expected["diagnostic_contains"]:
            assert text in str(error.value)
        assert observed.reads == []
        assert store.snapshots == {}
        assert observed.opened == [store.root_fd]
    assert Counter(observed.opened) == Counter(observed.closed)


@pytest.mark.parametrize(
    "capability",
    next(row for row in boundary_cases("snapshot") if row["id"] == "snapshot-missing-capability")[
        "setup"
    ]["capability_absence_each"],
)
def test_missing_capability(capability, tmp_path):
    path = tmp_path / "contracts/value.json"
    write_file(path, {"utf8": "7"})
    root = path.parent
    before = path.stat()
    with pytest.MonkeyPatch.context() as patch:
        words = capability.split()
        if len(words) == 1:
            patch.delattr(os, words[0])
        elif words[0] == "callable":
            patch.setattr(os, words[1], None)
        else:
            attribute = {
                "dir_fd": "supports_dir_fd",
                "follow_symlinks": "supports_follow_symlinks",
                "fd": "supports_fd",
            }[words[0]]
            patch.setattr(os, attribute, getattr(os, attribute) - {getattr(os, words[1])})
        with pytest.raises(InputAdmissionError) as error, observe_audit() as events:
            SnapshotStore(root)
        assert events == []
    assert str(error.value) == f"Input admission failed: capability: {root}"
    after = path.stat()
    assert (before.st_ino, before.st_size, before.st_mtime_ns) == (
        after.st_ino,
        after.st_size,
        after.st_mtime_ns,
    )
    assert path.read_bytes() == b"7"


def test_root_symlink(tmp_path):
    case = next(row for row in boundary_cases("snapshot") if row["id"] == "snapshot-root-symlink")
    real = tmp_path / "real"
    path = real / case["setup"]["path"]
    write_file(path, case["setup"])
    link = tmp_path / "contracts"
    link.symlink_to(path.parent, target_is_directory=True)
    with pytest.raises(InputAdmissionError, match="symlink"):
        SnapshotStore(link)
    assert path.read_bytes() == b"7"


def test_preflight_and_stream_are_lazy(tmp_path):
    path = tmp_path / "value.json"
    path.write_bytes(b"123")
    store = SnapshotStore(tmp_path)
    with observe_io(store) as observed:
        store.preflight(path)
        assert observed.reads == []
        assert store.snapshots == {}
        stream = SnapshotStream(store, str(path))
        assert not stream.loaded
        assert stream.read(1) == b"1"
        charge = store.requested_read_bytes
        assert stream.read() == b"23"
        assert store.requested_read_bytes == charge == 4
        stream.close()
        assert stream.closed
    assert Counter(observed.opened) == Counter(observed.closed)
    store.close()
    with pytest.raises(ValueError, match="closed"):
        store.preflight(path)
    with pytest.raises(ValueError, match="closed"):
        store.schema_paths()
    with pytest.raises(InputAdmissionError, match="outside-root"):
        store.canonical_path(tmp_path)


def test_fixture_inventory_is_unique():
    document = fixture_document("boundaries.json")
    assert len(document["cases"]) == 122
    assert len({row["id"] for row in document["cases"]}) == 122
    assert len(document["property_profile"]["properties"]) == 3
