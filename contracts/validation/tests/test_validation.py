"""Use literal semantic and format oracles with the actual effective validator."""

from __future__ import annotations

import inspect
import json
from io import BytesIO

import pytest
from check_jsonschema.checker import SchemaChecker
from check_jsonschema.formats import FormatOptions
from check_jsonschema.instance_loader import InstanceLoader
from check_jsonschema.parsers import ParserSet
from check_jsonschema.regex_variants import RegexImplementation, RegexVariantName
from check_jsonschema.reporter import REPORTER_BY_NAME, Reporter
from check_jsonschema.schema_loader import SchemaLoader, SchemaLoaderBase, SchemaParseError
from jsonschema import FormatChecker
from jsonschema.protocols import Validator
from support import fixture_document, materialize, validator_for

from arboresce_contract_validation.cli import LocalRegistryLoader, SnapshotStore, SnapshotStream


@pytest.mark.parametrize(
    "case", fixture_document("validation.json")["cases"], ids=lambda row: row["id"]
)
def test_literal_validation(case, tmp_path):
    materialize(
        tmp_path,
        {
            "contracts/entry.schema.json": {"json": case["schema"]},
            "contracts/value.json": {"json": case["instance"]},
        },
    )
    with SnapshotStore(tmp_path / "contracts") as store:
        loader = LocalRegistryLoader(store, str(tmp_path / "contracts/entry.schema.json"))
        regex = RegexImplementation(RegexVariantName.default)
        options = FormatOptions(regex_impl=regex)
        template = loader.entry_loader.get_validator(
            loader.entry_loader.entry, case["instance"], options, regex, False
        )
        validator = loader.get_validator(
            loader.entry_loader.entry, case["instance"], options, regex, False
        )
        assert isinstance(template, Validator)
        assert isinstance(template.format_checker, FormatChecker)
        assert validator is not template
        assert type(validator) is type(template)
        assert validator.schema is template.schema
        assert template.schema == case["schema"]
        assert loader.entry_loader.validator_class is None
        assert isinstance(loader.entry_loader.get_schema(), dict)
        assert validator._registry is not template._registry
        assert validator.format_checker is template.format_checker
        assert validator.VALIDATORS["pattern"] is template.VALIDATORS["pattern"]
        assert validator.VALIDATORS["patternProperties"] is template.VALIDATORS["patternProperties"]
        errors = list(validator.iter_errors(case["instance"]))
        assert (not errors) is case["expected_valid"]
        assert (errors[0].validator if errors else None) == case["expected_keyword"]
        assert loader.preload_error is None
        assert validator_for(loader, case["instance"]) is validator
        assert loader.entry_loader.get_schema() is loader.entry_loader.get_schema()
        assert (
            loader.entry_loader.get_schema_retrieval_uri()
            == (tmp_path / "contracts/entry.schema.json").as_uri()
        )
        if case["id"] == "complete-error-order":
            assert [list(error.path) for error in errors] == [["left"], ["right"]]
            assert [error.message for error in errors] == [
                "'x' is not of type 'integer'",
                "2 is not of type 'boolean'",
            ]
        assert json.loads(store.snapshots[loader.entry_loader.entry].data) == case["schema"]


def test_disabled_format_preserves_template(tmp_path):
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "string",
        "format": "date",
    }
    materialize(tmp_path, {"contracts/entry.schema.json": {"json": schema}})
    with SnapshotStore(tmp_path / "contracts") as store:
        loader = LocalRegistryLoader(store, str(tmp_path / "contracts/entry.schema.json"))
        regex = RegexImplementation(RegexVariantName.default)
        options = FormatOptions(regex_impl=regex, enabled=False)
        template = loader.entry_loader.get_validator(
            loader.entry_loader.entry, "2024-02-30", options, regex, False
        )
        validator = loader.get_validator(
            loader.entry_loader.entry, "2024-02-30", options, regex, False
        )
        assert template.format_checker is validator.format_checker is None
        assert type(validator) is type(template)
        assert validator.is_valid("2024-02-30")


def test_pinned_parser_binary_interfaces():
    payload = b'{"value": 1}'
    parser = ParserSet()
    assert parser.parse_data_with_path(payload, "contracts/value.json", "json", "json") == {
        "value": 1
    }
    with BytesIO(payload) as stream:
        assert parser.parse_data_with_path(stream, "contracts/value.json", "json", "json") == {
            "value": 1
        }
    stream = BytesIO(payload)
    stream.name = "contracts/value.json"
    values = list(InstanceLoader([stream], "json", "json").iter_files())
    assert values == [("contracts/value.json", {"value": 1})]
    assert stream.closed
    error = SchemaParseError("contracts/entry.schema.json")
    assert isinstance(error, ValueError)
    assert str(error) == "contracts/entry.schema.json"


API_FIXTURES = {
    case["id"]: case["expected"] for case in fixture_document("upstream-api.json")["cases"]
}


def test_pinned_checker_call_binding():
    expected = API_FIXTURES["schema-checker-call-binding"]
    signature = inspect.signature(SchemaChecker.__init__)
    parameters = signature.parameters
    assert [p.name for p in parameters.values() if p.kind is p.POSITIONAL_OR_KEYWORD] == expected[
        "positional"
    ]
    assert [p.name for p in parameters.values() if p.kind is p.KEYWORD_ONLY] == expected[
        "keyword_only"
    ]
    assert not any(p.kind in (p.VAR_POSITIONAL, p.VAR_KEYWORD) for p in parameters.values())
    assert parameters["traceback_mode"].default == expected["traceback_default"]
    assert parameters["fill_defaults"].default is expected["fill_defaults_default"]
    # Binding inspects call shape only; real checker execution has separate cases below.
    signature.bind(
        None,
        None,
        None,
        None,
        format_opts=None,
        regex_impl=None,
        traceback_mode="short",
        fill_defaults=False,
    )


def test_pinned_schema_loader_call_binding():
    expected = API_FIXTURES["schema-loader-call-binding"]
    signature = inspect.signature(SchemaLoader.__init__)
    assert [
        p.name for p in signature.parameters.values() if p.kind is p.POSITIONAL_OR_KEYWORD
    ] == expected["init_positional"]
    assert [p.name for p in signature.parameters.values() if p.kind is p.KEYWORD_ONLY] == expected[
        "init_keyword_only"
    ]
    assert {
        name: signature.parameters[name].default for name in expected["init_defaults"]
    } == expected["init_defaults"]
    signature.bind(None, "contracts/entry.schema.json", disable_cache=True)
    for owner in (SchemaLoaderBase, SchemaLoader):
        method = inspect.signature(owner.get_validator)
        assert list(method.parameters) == expected["validator_positional"]
        assert all(p.kind is p.POSITIONAL_OR_KEYWORD for p in method.parameters.values())
        method.bind(None, "contracts/value.json", {}, None, None, False)


def test_pinned_instance_loader_call_binding():
    expected = API_FIXTURES["instance-loader-call-binding"]
    signature = inspect.signature(InstanceLoader.__init__)
    assert list(signature.parameters)[:4] == expected["positional_prefix"]
    assert {name: signature.parameters[name].default for name in expected["defaults"]} == expected[
        "defaults"
    ]
    binding = signature.bind(None, [], default_filetype="json", force_filetype="json")
    assert "data_transform" not in binding.arguments


def test_pinned_parser_call_binding():
    expected = API_FIXTURES["parser-call-binding"]
    inspect.signature(ParserSet).bind()
    signature = inspect.signature(ParserSet.parse_data_with_path)
    assert list(signature.parameters) == expected["parse_positional"]
    assert all(p.kind is p.POSITIONAL_OR_KEYWORD for p in signature.parameters.values())
    assert signature.parameters["force_filetype"].default is expected["force_filetype_default"]
    signature.bind(
        None, b"7", "contracts/value.json", default_filetype="json", force_filetype="json"
    )


def test_pinned_format_options_call_binding():
    expected = API_FIXTURES["format-options-call-binding"]
    signature = inspect.signature(FormatOptions.__init__)
    assert [
        p.name for p in signature.parameters.values() if p.kind is p.POSITIONAL_OR_KEYWORD
    ] == expected["positional"]
    assert [p.name for p in signature.parameters.values() if p.kind is p.KEYWORD_ONLY] == expected[
        "keyword_only"
    ]
    assert signature.parameters["enabled"].default is expected["enabled_default"]
    assert signature.parameters["disabled_formats"].default == tuple(
        expected["disabled_formats_default"]
    )
    regex = RegexImplementation(RegexVariantName.default)
    options = FormatOptions(regex_impl=regex)
    assert set(vars(options)) == set(expected["attributes"])
    assert options.regex_impl is regex and options.enabled is expected["enabled_default"]
    assert options.disabled_formats == tuple(expected["disabled_formats_default"])


def test_pinned_regex_enum_call_binding():
    expected = API_FIXTURES["regex-enum-call-binding"]
    assert {member.name: member.value for member in RegexVariantName} == expected["members"]
    signature = inspect.signature(RegexImplementation.__init__)
    assert list(signature.parameters) == expected["constructor_positional"]
    assert all(p.kind is p.POSITIONAL_OR_KEYWORD for p in signature.parameters.values())
    regex = RegexImplementation(RegexVariantName.default)
    assert regex.variant is RegexVariantName.default


def test_pinned_reporter_call_binding():
    expected = API_FIXTURES["reporter-call-binding"]
    assert sorted(REPORTER_BY_NAME) == expected["keys"]
    assert {name: cls.__name__ for name, cls in REPORTER_BY_NAME.items()} == expected["classes"]
    assert inspect.isabstract(Reporter) is expected["Reporter_is_abstract"]
    for cls in REPORTER_BY_NAME.values():
        assert issubclass(cls, Reporter)
        inspect.signature(cls).bind(verbosity=1)
        assert cls(verbosity=1).verbosity == 1
    assert list(inspect.signature(Reporter.report_result).parameters) == [
        "self",
        expected["report_result_parameter"],
    ]


@pytest.mark.parametrize(
    "case_id, expected_exit", [("integer-positive", 0), ("integer-negative", 1)]
)
def test_pinned_checker_returns_actual_integer_status(case_id, expected_exit, tmp_path, capsys):
    (case,) = [row for row in fixture_document("validation.json")["cases"] if row["id"] == case_id]
    materialize(
        tmp_path,
        {
            "contracts/entry.schema.json": {"json": case["schema"]},
            "contracts/value.json": {"json": case["instance"]},
        },
    )
    with SnapshotStore(tmp_path / "contracts") as store:
        schema = str(tmp_path / "contracts/entry.schema.json")
        stream = SnapshotStream(store, str(tmp_path / "contracts/value.json"))
        regex = RegexImplementation(RegexVariantName.default)
        checker = SchemaChecker(
            LocalRegistryLoader(store, schema),
            InstanceLoader([stream], default_filetype="json", force_filetype="json"),
            REPORTER_BY_NAME["text"](verbosity=1),
            format_opts=FormatOptions(regex_impl=regex),
            regex_impl=regex,
            traceback_mode="short",
            fill_defaults=False,
        )
        status = checker.run()
        assert type(status) is int and status == expected_exit
        assert stream.closed
    output = capsys.readouterr()
    assert output.err == ""
    assert ("ok -- validation done" in output.out) is (expected_exit == 0)
    assert inspect.getsourcefile(SchemaChecker).endswith("check_jsonschema/checker.py")
