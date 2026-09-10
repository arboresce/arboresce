"""Consumed interfaces for check-jsonschema 0.38.0; review when its pin changes."""

from pathlib import Path
from typing import IO, Any

class ParserSet:
    def __init__(self) -> None: ...
    def parse_data_with_path(
        self,
        data: IO[bytes] | bytes,
        path: Path | str,
        default_filetype: str,
        force_filetype: str | None = None,
    ) -> Any: ...
