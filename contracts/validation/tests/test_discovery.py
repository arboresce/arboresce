"""Real directory, entry and depth bounds, independently of schema payload reads."""

from collections import Counter

import pytest
from support import boundary_cases, observe_discovery, observe_io

from arboresce_contract_validation import cli


@pytest.mark.parametrize("case", boundary_cases("discovery"), ids=lambda row: row["id"])
def test_discovery_bounds(case, tmp_path):
    setup, expected = case["setup"], case["expected"]
    root = tmp_path / "contracts"
    root.mkdir()
    for row in setup.get("directories", []):
        for index in range(row["index_start"], row["index_end"] + 1):
            (root / row["name_template"].format(index=index)).mkdir()
    for row in setup.get("files", []):
        for index in range(row["index_start"], row["index_end"] + 1):
            (root / row["name_template"].format(index=index)).write_text(row["utf8"])
    leaf = root
    for _ in range(setup.get("chain_directory_count", 0)):
        leaf = leaf / setup["chain_component"]
        leaf.mkdir()
    if "leaf_file" in setup:
        (leaf / setup["leaf_file"]["name"]).write_text(setup["leaf_file"]["utf8"])
    store = cli.SnapshotStore(root)
    with observe_io(store) as observed, observe_discovery() as discovery:
        if expected.get("classification") == "admission":
            with pytest.raises(cli.InputAdmissionError) as error:
                store.schema_paths()
            for text in expected["diagnostic_contains"]:
                assert text.replace("ROOT", str(root)) in str(error.value)
        else:
            paths = store.schema_paths()
            assert [str(path.relative_to(root)) for path in paths] == expected.get(
                "returned_relative_paths", expected.get("returned_paths")
            )
        assert observed.reads == []
        if "entered_directories_including_root" in expected:
            assert len(discovery.scans) == expected["entered_directories_including_root"]
        if "scandir_calls" in expected:
            assert len(discovery.scans) == expected["scandir_calls"]
        if "enumerated_entries" in expected:
            assert max(discovery.enumerated) == expected["enumerated_entries"]
        if "maximum_depth" in expected:
            assert max(discovery.depths) == expected["maximum_depth"]
        if "buffered_names_max" in expected:
            assert max(discovery.buffers) == expected["buffered_names_max"]
        if "names_sort_calls" in expected:
            assert discovery.sorts == expected["names_sort_calls"]
        if case["id"] == "discovery-depth-cap-plus-one":
            assert len(observed.opened) == 65
            assert leaf not in discovery.entered
            assert "leaf.schema.json" not in discovery.stats
        if case["id"] == "discovery-directory-cap-plus-one":
            assert len(discovery.scans) == len(discovery.entered) - 1
    assert Counter(observed.opened) == Counter(observed.closed)
