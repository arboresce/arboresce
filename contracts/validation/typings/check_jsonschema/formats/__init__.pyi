"""Consumed interfaces for check-jsonschema 0.38.0; review when its pin changes."""

from ..regex_variants import RegexImplementation

class FormatOptions:
    enabled: bool
    regex_impl: RegexImplementation
    disabled_formats: tuple[str, ...]
    def __init__(
        self,
        *,
        regex_impl: RegexImplementation,
        enabled: bool = True,
        disabled_formats: tuple[str, ...] = (),
    ) -> None: ...
