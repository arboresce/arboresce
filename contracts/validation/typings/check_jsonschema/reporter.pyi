"""Consumed interfaces for check-jsonschema 0.38.0; review when its pin changes."""

from abc import ABC, abstractmethod
from typing import Any

from .result import CheckResult

class Reporter(ABC):
    verbosity: int
    def __init__(self, *, verbosity: int, **kwargs: Any) -> None: ...
    @abstractmethod
    def report_success(self, result: CheckResult) -> None: ...
    @abstractmethod
    def report_errors(self, result: CheckResult) -> None: ...
    def report_result(self, result: CheckResult) -> None: ...

REPORTER_BY_NAME: dict[str, type[Reporter]]
