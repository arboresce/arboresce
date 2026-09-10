"""One reproducible, bounded property profile; no ambient plugin configuration."""

import importlib.metadata
import os
import re
import sys
from pathlib import Path

import pytest
from hypothesis import settings

# The locked upstream release still uses Click's compatibility alias. Assert
# its diagnostic rather than weakening the warnings-as-errors policy.
assert "check_jsonschema" not in sys.modules
with pytest.warns(
    DeprecationWarning,
    match=r"\A"
    + re.escape("'click.utils.LazyFile' is deprecated and will be removed in Click 9.0.")
    + r"\Z",
) as dependency_warnings:
    from support import suite_binding, write_receipt

assert len(dependency_warnings) == 1
dependency_warning = dependency_warnings[0]
assert dependency_warning.category is DeprecationWarning
assert dependency_warning.filename == str(
    importlib.metadata.distribution("check-jsonschema").locate_file(
        "check_jsonschema/cli/param_types.py"
    )
)
assert dependency_warning.lineno == 126

settings.register_profile(
    "contract_validation_bounded_v1",
    max_examples=64,
    deadline=200,
    database=None,
    print_blob=True,
)
settings.load_profile("contract_validation_bounded_v1")


def pytest_sessionstart(session: pytest.Session) -> None:
    receipt = suite_binding()
    receipt.update(
        argv=sys.argv,
        original_argv=sys.orig_argv,
        cwd=os.getcwd(),
        environment=dict(os.environ),
        collection_only=session.config.option.collectonly,
        dependency_import_warnings=[
            {
                "category": dependency_warning.category.__name__,
                "message": str(dependency_warning.message),
                "filename": dependency_warning.filename,
                "lineno": dependency_warning.lineno,
            }
        ],
    )
    write_receipt(Path(os.environ["ARBORESCE_TEST_SUITE_RECEIPT"]), receipt)
