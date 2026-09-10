"""Consumed interfaces for check-jsonschema 0.38.0; review when its pin changes."""

from enum import Enum

class RegexVariantName(Enum):
    default = "default"
    nonunicode = "nonunicode"
    python = "python"

class RegexImplementation:
    variant: RegexVariantName
    def __init__(self, variant: RegexVariantName) -> None: ...
