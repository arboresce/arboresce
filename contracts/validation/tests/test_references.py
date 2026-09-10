"""Reference interpretation, immutable aliases and fresh invocation behavior."""

from __future__ import annotations

import json
from collections import Counter

import pytest
from referencing.exceptions import NoSuchResource
from support import (
    boundary_cases,
    fail_reference_parse,
    materialize,
    observe_audit,
    observe_io,
    validator_for,
)

from arboresce_contract_validation.cli import (
    InputAdmissionError,
    LocalRegistryLoader,
    SnapshotStore,
)


@pytest.mark.parametrize(
    "case", boundary_cases("reference", "resolution"), ids=lambda row: row["id"]
)
def test_reference_interpretation(case, tmp_path):
    setup, expected = case["setup"], case["expected"]
    materialize(tmp_path, setup["files"])
    instance = json.loads((tmp_path / setup["instances"][0]).read_bytes())
    with SnapshotStore(tmp_path / "contracts") as store:
        loader = LocalRegistryLoader(store, str(tmp_path / setup["entry"]))
        if expected.get("classification") == "admission":
            with pytest.raises(InputAdmissionError) as error:
                validator_for(loader, instance)
            for text in expected["diagnostic_contains"]:
                assert text in str(error.value)
            assert loader.preload_error is None
            assert loader.validator is None
        else:
            validator = validator_for(loader, instance)
            if expected.get("classification") == "infrastructure":
                with pytest.raises(Exception) as error:
                    list(validator.iter_errors(instance))
                assert not isinstance(error.value, InputAdmissionError)
            else:
                errors = list(validator.iter_errors(instance))
                assert (not errors) is (expected["exit"] == 0)
                if "error_keywords" in expected:
                    assert [error.validator for error in errors] == expected["error_keywords"]
                if "error_paths" in expected:
                    assert [list(error.path) for error in errors] == expected["error_paths"]
                if "error_messages" in expected:
                    assert [error.message for error in errors] == expected["error_messages"]
                if "keyword" in expected:
                    assert [error.validator for error in errors] == [expected["keyword"]]
                for wrong in expected.get("other_definition_values_forbidden", []):
                    assert [error.validator for error in validator.iter_errors(wrong)] == ["const"]
            if "registry_path" in expected:
                assert ("lazy" if loader.preload_error is not None else "optimized") == (
                    expected["registry_path"]
                )
            if "eager_cause_type" in expected:
                assert type(loader.preload_error).__name__ == expected["eager_cause_type"]
            if "destination" in expected:
                target = tmp_path / expected["destination"]
                resource = loader.retrieve(target.as_uri())
                value = resource.contents
                for component in expected.get("pointer", []):
                    value = value[component]
                if "resolved_const" in expected:
                    assert value["const"] == expected["resolved_const"]
            for alias in expected.get("alias_pair", []):
                uri = alias.replace("<ROOT_FILE_URI>", tmp_path.as_uri())
                assert loader.aliases[uri].path == tmp_path / expected["destination"]
                assert loader.retrieve(uri).contents == {"const": expected["resolved_const"]}


@pytest.mark.parametrize("case", boundary_cases("fallback-boundary"), ids=lambda row: row["id"])
def test_process_controls_cannot_select_lazy_fallback(case, tmp_path):
    setup = case["setup"]
    materialize(tmp_path, setup["files"])
    failure = {
        "MemoryError": MemoryError(),
        "deadline-exhausted": TimeoutError("deadline-exhausted"),
        "KeyboardInterrupt": KeyboardInterrupt(),
        "SystemExit": SystemExit(7),
    }[case["expected"]["propagated"]]
    store = SnapshotStore(tmp_path / "contracts")
    loader = LocalRegistryLoader(store, str(tmp_path / setup["entry"]))
    with observe_io(store) as observed, fail_reference_parse(loader, failure) as parsed:
        with pytest.raises(type(failure)) as error:
            validator_for(loader, 7)
        assert error.value is failure
        assert parsed == [(tmp_path / "contracts/good.schema.json", {"const": 7})]
        assert loader.preload_error is None
        assert loader.validator is None
    assert Counter(observed.opened) == Counter(observed.closed)


def test_alias_collision_preserves_original(tmp_path):
    (case,) = boundary_cases("alias-admission")
    materialize(tmp_path, case["setup"]["files"])
    with SnapshotStore(tmp_path / "contracts") as store:
        loader = LocalRegistryLoader(store, str(tmp_path / "contracts/a.schema.json"))
        first = store.read(tmp_path / "contracts/a.schema.json")
        second = store.read(tmp_path / "contracts/b.schema.json")
        loader.register_alias("a.schema.json", first)
        loader.register_alias("a.schema.json", first)
        with pytest.raises(InputAdmissionError) as error:
            loader.register_alias("a.schema.json", second)
        for text in case["expected"]["diagnostic_contains"]:
            assert text in str(error.value)
        assert loader.aliases == {"a.schema.json": first}
        assert loader.validator is None


@pytest.mark.parametrize("case", boundary_cases("retrieval"), ids=lambda row: row["id"])
def test_retrieval_is_closed_over_admitted_aliases(case, tmp_path):
    setup = case["setup"]
    materialize(tmp_path, setup["files"])
    outside = tmp_path / "outside.schema.json"
    outside.write_text('{"const":7}')
    with SnapshotStore(tmp_path / "contracts") as store:
        loader = LocalRegistryLoader(store, str(tmp_path / setup["entry"]))
        validator_for(loader, 7)
        before = dict(store.snapshots)
        charge = store.requested_read_bytes
        reference = setup["files"][setup["entry"]]["json"]["$ref"]
        with pytest.raises(NoSuchResource) as error, observe_audit() as events:
            loader.retrieve(reference)
        assert events == []
        assert error.value.ref == reference
        assert store.snapshots == before
        assert store.requested_read_bytes == charge
        assert outside not in store.snapshots


def test_new_invocation_reads_repaired_and_restored_bytes(tmp_path):
    (case,) = boundary_cases("freshness")
    setup = case["setup"]
    materialize(tmp_path, setup["files"])
    library = tmp_path / "contracts/lib.schema.json"
    original = library.read_bytes()
    outputs = []
    contexts = []
    for data in (original, b'{"const":18}', original):
        library.write_bytes(data)
        with SnapshotStore(tmp_path / "contracts") as store:
            loader = LocalRegistryLoader(store, str(tmp_path / setup["entry"]))
            contexts.append(loader)
            errors = list(validator_for(loader, 17).iter_errors(17))
            outputs.append(1 if errors else 0)
            if data != original:
                assert [error.validator for error in errors] == [case["expected"]["middle_keyword"]]
                assert [error.message for error in errors] == [case["expected"]["middle_message"]]
    assert outputs == case["expected"]["exits"]
    assert len({id(context) for context in contexts}) == 3
    assert library.read_bytes() == original
