"""Consumed interfaces for check-jsonschema 0.38.0; review when its pin changes."""

from typing import Literal

from .formats import FormatOptions
from .instance_loader import InstanceLoader
from .regex_variants import RegexImplementation
from .reporter import Reporter
from .schema_loader import SchemaLoaderBase

class SchemaChecker:
    def __init__(
        self,
        schema_loader: SchemaLoaderBase,
        instance_loader: InstanceLoader,
        reporter: Reporter,
        *,
        format_opts: FormatOptions,
        regex_impl: RegexImplementation,
        traceback_mode: Literal["minimal", "short", "full"] = "short",
        fill_defaults: bool = False,
    ) -> None: ...
    def run(self) -> int: ...
