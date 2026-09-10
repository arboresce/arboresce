"""Consumed interfaces for check-jsonschema 0.38.0; review when its pin changes."""

from pathlib import Path
from typing import Any

from jsonschema.protocols import Validator

from ..formats import FormatOptions
from ..regex_variants import RegexImplementation

class SchemaParseError(ValueError): ...

class SchemaLoaderBase:
    def get_validator(
        self,
        path: Path | str,
        instance_doc: dict[str, Any],
        format_opts: FormatOptions,
        regex_impl: RegexImplementation,
        fill_defaults: bool,
    ) -> Validator: ...

class SchemaLoader(SchemaLoaderBase):
    def __init__(
        self,
        schemafile: str,
        *,
        base_uri: str | None = None,
        validator_class: type[Validator] | None = None,
        disable_cache: bool = True,
    ) -> None: ...
    def get_schema_retrieval_uri(self) -> str | None: ...
    def get_schema(self) -> dict[str, Any]: ...
