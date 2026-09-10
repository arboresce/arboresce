"""Consumed interfaces for check-jsonschema 0.38.0; review when its pin changes."""

from collections.abc import Sequence
from typing import IO

class InstanceLoader:
    def __init__(
        self,
        files: Sequence[IO[bytes]],
        default_filetype: str = "json",
        force_filetype: str | None = None,
    ) -> None: ...
