"""Bounded generated paths and payloads supplement the fixed independent oracles."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path
from urllib.parse import quote

from hypothesis import given
from hypothesis import strategies as st
from support import fixture_document, materialize, validator_for

from arboresce_contract_validation.cli import LocalRegistryLoader, SnapshotStore

PROFILE = fixture_document("boundaries.json")["property_profile"]
SEGMENT = st.text(alphabet=PROFILE["alphabet"], min_size=1, max_size=12)
TAIL = st.text(alphabet=PROFILE["alphabet"], min_size=0, max_size=11)
PARENTS = st.lists(SEGMENT, min_size=0, max_size=3)
PAYLOAD = st.binary(min_size=0, max_size=4096)


@given(parents=PARENTS, tail=TAIL, payload=PAYLOAD)
def test_property_alias_roundtrip_separation(parents, tail, payload):
    with tempfile.TemporaryDirectory(prefix="registry-alias-") as directory:
        root = Path(directory)
        first = Path("contracts", *parents, "a" + tail + ".schema.json")
        second = Path("contracts", *parents, "b" + tail + ".schema.json")
        first_value = {"const": ["a", payload.hex()]}
        second_value = {"const": ["b", payload.hex()]}
        materialize(
            root,
            {
                "contracts/entry.schema.json": {"json": {}},
                str(first): {"json": first_value},
                str(second): {"json": second_value},
            },
        )
        original = {first: (root / first).read_bytes(), second: (root / second).read_bytes()}
        with SnapshotStore(root / "contracts") as store:
            loader = LocalRegistryLoader(store, str(root / "contracts/entry.schema.json"))
            loader.prepare_registry()
            for relative, value in ((first, first_value), (second, second_value)):
                actual = root / relative
                alias = quote(relative.relative_to("contracts").as_posix(), safe="/")
                assert loader.aliases[alias].path == actual
                assert loader.aliases[alias].data == original[relative]
                assert loader.aliases[actual.as_uri()] is loader.aliases[alias]
                assert loader.retrieve(alias).contents == value
                assert loader.retrieve(actual.as_uri()).contents == value
            assert (
                loader.aliases[(root / first).as_uri()]
                is not (loader.aliases[(root / second).as_uri()])
            )


@given(parents=PARENTS, tail=TAIL)
def test_property_relocation(parents, tail):
    with tempfile.TemporaryDirectory(prefix="registry-relocation-") as directory:
        base = Path(directory)
        destination = Path("contracts", *parents, "a" + tail + ".schema.json")
        reference = quote(destination.relative_to("contracts").as_posix(), safe="/")
        files = {
            "contracts/entry.schema.json": {"json": {"$ref": reference}},
            "contracts/value.json": {"json": 17},
            str(destination): {"json": {"const": 17}},
        }
        first, second = base / "original", base / "relocated"
        materialize(first, files)
        materialize(second, files)
        identities = []
        for root in (first, second):
            with SnapshotStore(root / "contracts") as store:
                loader = LocalRegistryLoader(store, str(root / "contracts/entry.schema.json"))
                validator = validator_for(loader, 17)
                assert list(validator.iter_errors(17)) == []
                assert [error.validator for error in validator.iter_errors(18)] == ["const"]
                assert loader.retrieve(reference).contents == {"const": 17}
                assert loader.aliases[reference].path == root / destination
                identities.append(loader.aliases[reference].path.as_uri())
        assert identities == [(first / destination).as_uri(), (second / destination).as_uri()]
        assert identities[0] != identities[1]
        for name in files:
            assert (first / name).read_bytes() == (second / name).read_bytes()


@given(payload=PAYLOAD)
def test_property_snapshot_conservation(payload):
    with tempfile.TemporaryDirectory(prefix="registry-snapshot-") as directory:
        root = Path(directory)
        path = root / "value.json"
        path.write_bytes(payload)
        with SnapshotStore(root) as store:
            snapshot = store.read(path)
            assert snapshot.data == payload
            assert len(snapshot.data) == len(payload)
            assert store.requested_read_bytes == len(payload) + 1
            assert len(store.snapshots) == 1
            assert store.read(path) is snapshot
            assert store.requested_read_bytes == len(payload) + 1


def test_property_profile_matches_fixture_contract():
    from hypothesis import settings

    assert settings.default.max_examples == PROFILE["max_examples"]
    assert settings.default.deadline.total_seconds() * 1000 == PROFILE["deadline_ms"]
    assert settings.default.database is PROFILE["database"] is None
    assert settings.default.print_blob is PROFILE["print_blob"] is True
    assert json.loads(json.dumps(PROFILE))["seed_option"] == "--hypothesis-seed=0"
