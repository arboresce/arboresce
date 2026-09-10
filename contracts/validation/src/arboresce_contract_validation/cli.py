"""Validate local JSON contracts with bounded snapshots and a fresh resource registry."""

from __future__ import annotations

import io
import os
import stat
from collections.abc import Generator
from contextlib import ExitStack, contextmanager
from dataclasses import dataclass
from operator import index
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal, Protocol, SupportsIndex, cast
from urllib.parse import quote, urlsplit

import click
from check_jsonschema.checker import SchemaChecker
from check_jsonschema.formats import FormatOptions
from check_jsonschema.instance_loader import InstanceLoader
from check_jsonschema.parsers import ParserSet
from check_jsonschema.regex_variants import RegexImplementation, RegexVariantName
from check_jsonschema.reporter import REPORTER_BY_NAME
from check_jsonschema.schema_loader import SchemaLoader, SchemaLoaderBase, SchemaParseError
from jsonschema.protocols import Validator
from referencing import Registry, Resource
from referencing.exceptions import NoSuchResource
from referencing.jsonschema import DRAFT202012

if TYPE_CHECKING:
    from jsonschema import FormatChecker

    class _ConfiguredValidator(Validator, Protocol):
        format_checker: FormatChecker | None


MAX_FILE_BYTES = 64 * 1024 * 1024
MAX_FILES = 1280
MAX_DIRECTORY_ENTRIES = 2 * MAX_FILES
MAX_SCHEMA_DEPTH = 64
MAX_READ_BYTES = 512 * 1024 * 1024
READ_CHUNK_BYTES = 64 * 1024


class InputAdmissionError(RuntimeError):
    """A filesystem or resource bound failed before validation could complete."""

    def __init__(self, reason: str, path: Path | str) -> None:
        self.reason, self.path = reason, path
        super().__init__(f"Input admission failed: {reason}: {path}")


def file_identity(value: os.stat_result) -> tuple[int, ...]:
    return (
        value.st_dev,
        value.st_ino,
        value.st_mode,
        value.st_size,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )


def require_same(before: os.stat_result, after: os.stat_result, path: Path) -> None:
    if file_identity(before) != file_identity(after):
        raise InputAdmissionError("changed", path)


def require_kind(value: os.stat_result, path: Path, *, directory: bool = False) -> None:
    if stat.S_ISLNK(value.st_mode):
        raise InputAdmissionError("symlink", path)
    if not (stat.S_ISDIR(value.st_mode) if directory else stat.S_ISREG(value.st_mode)):
        raise InputAdmissionError("not-regular", path)


@dataclass(frozen=True)
class Snapshot:
    path: Path
    data: bytes


class SnapshotStore:
    """Own descriptors and immutable bytes for exactly one contract invocation."""

    def __init__(self, root: Path) -> None:
        self.root = Path(os.path.abspath(root))
        self.snapshots: dict[Path, Snapshot] = {}
        self.requested_read_bytes = 0
        self.root_fd: int | None = None
        if (
            not all(
                type(getattr(os, name, None)) is int
                for name in ("O_NOFOLLOW", "O_NONBLOCK", "O_DIRECTORY", "O_CLOEXEC")
            )
            or not all(
                callable(getattr(os, name, None))
                for name in ("open", "stat", "fstat", "read", "close", "scandir")
            )
            or os.open not in os.supports_dir_fd
            or os.stat not in os.supports_dir_fd
            or os.stat not in os.supports_follow_symlinks
            or os.scandir not in os.supports_fd
        ):
            raise InputAdmissionError("capability", self.root)
        before = self.root.stat(follow_symlinks=False)
        require_kind(before, self.root, directory=True)
        fd = os.open(self.root, self.open_flags | os.O_DIRECTORY)
        try:
            require_same(before, os.fstat(fd), self.root)
            require_same(before, self.root.stat(follow_symlinks=False), self.root)
        except BaseException:
            os.close(fd)
            raise
        self.root_fd = fd

    @property
    def open_flags(self) -> int:
        return os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC

    def close(self) -> None:
        fd, self.root_fd = self.root_fd, None
        if fd is not None:
            os.close(fd)

    def __enter__(self) -> SnapshotStore:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def canonical_path(self, filename: Path | str) -> Path:
        path = Path(os.path.abspath(filename))
        if not path.is_relative_to(self.root) or path == self.root:
            raise InputAdmissionError("outside-root", filename)
        return path

    @contextmanager
    def opened(self, filename: Path | str) -> Generator[tuple[int, os.stat_result]]:
        path = self.canonical_path(filename)
        if self.root_fd is None:
            raise ValueError("snapshot store is closed")
        handles = ExitStack()
        parents: list[tuple[int, str, os.stat_result, int, Path]] = []
        parent_fd, parent_path = self.root_fd, self.root
        try:
            root_before = os.fstat(self.root_fd)
            require_same(root_before, self.root.stat(follow_symlinks=False), self.root)
            parts = path.relative_to(self.root).parts
            actual = root_before
            for index, name in enumerate(parts):
                current_path = parent_path / name
                before = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
                is_directory = index < len(parts) - 1
                require_kind(before, current_path, directory=is_directory)
                flags = self.open_flags | (os.O_DIRECTORY if is_directory else 0)
                fd = os.open(name, flags, dir_fd=parent_fd)
                handles.callback(os.close, fd)
                actual = os.fstat(fd)
                require_kind(actual, current_path, directory=is_directory)
                require_same(before, actual, current_path)
                parents.append((parent_fd, name, before, fd, current_path))
                parent_fd, parent_path = fd, current_path
            yield parent_fd, actual
            for parent, name, before, fd, current_path in reversed(parents):
                require_same(before, os.fstat(fd), current_path)
                require_same(
                    before,
                    os.stat(name, dir_fd=parent, follow_symlinks=False),
                    current_path,
                )
            require_same(root_before, os.fstat(self.root_fd), self.root)
            require_same(root_before, self.root.stat(follow_symlinks=False), self.root)
        except InputAdmissionError as error:
            if error.path != path:
                raise InputAdmissionError(
                    f"{error.reason} (component {error.path})", path
                ) from error
            raise
        except OSError as error:
            if error.filename is not None:
                error.add_note(f"Filesystem component: {error.filename}")
                error.filename = str(path)
            raise
        finally:
            handles.close()

    def preflight(self, filename: Path | str) -> None:
        """Preserve ordered argument admission without reading or caching payload."""
        with self.opened(filename):
            pass

    def read(self, filename: Path | str) -> Snapshot:
        path = self.canonical_path(filename)
        if path in self.snapshots:
            return self.snapshots[path]
        if len(self.snapshots) >= MAX_FILES:
            raise InputAdmissionError("file-count", path)
        with self.opened(path) as (fd, before):
            if before.st_size > MAX_FILE_BYTES:
                raise InputAdmissionError("file-limit", path)
            chunks: list[bytes] = []
            count = 0
            while True:
                requested = (
                    min(READ_CHUNK_BYTES, before.st_size - count) if count < before.st_size else 1
                )
                if self.requested_read_bytes + requested > MAX_READ_BYTES:
                    raise InputAdmissionError("aggregate-limit", path)
                self.requested_read_bytes += requested
                block = os.read(fd, requested)
                if not block:
                    break
                count += len(block)
                if count > before.st_size:
                    raise InputAdmissionError("changed", path)
                chunks.append(block)
            if count != before.st_size:
                raise InputAdmissionError("changed", path)
            data = b"".join(chunks)
        snapshot = Snapshot(path, data)
        self.snapshots[path] = snapshot
        return snapshot

    def schema_paths(self) -> list[Path]:
        """Discover regular schema paths without following a directory symlink."""
        if self.root_fd is None:
            raise ValueError("snapshot store is closed")
        paths: list[Path] = []
        directories = 0
        entry_count = 0

        def visit(fd: int, path: Path, depth: int) -> None:
            nonlocal directories, entry_count
            directories += 1
            if directories > MAX_FILES:
                raise InputAdmissionError("file-count", path)
            before = os.fstat(fd)
            with os.scandir(fd) as entries:
                names: list[str] = []
                for entry in entries:
                    entry_count += 1
                    if entry_count > MAX_DIRECTORY_ENTRIES:
                        raise InputAdmissionError("file-count", path)
                    names.append(entry.name)
            names.sort()
            for name in names:
                child = path / name
                info = os.stat(name, dir_fd=fd, follow_symlinks=False)
                if stat.S_ISDIR(info.st_mode):
                    if depth >= MAX_SCHEMA_DEPTH:
                        raise InputAdmissionError("depth-limit", child)
                    child_fd = os.open(name, self.open_flags | os.O_DIRECTORY, dir_fd=fd)
                    try:
                        require_same(info, os.fstat(child_fd), child)
                        visit(child_fd, child, depth + 1)
                        require_same(info, os.fstat(child_fd), child)
                        require_same(info, os.stat(name, dir_fd=fd, follow_symlinks=False), child)
                    finally:
                        os.close(child_fd)
                elif name.endswith(".schema.json"):
                    require_kind(info, child)
                    paths.append(child)
                    if len(paths) > MAX_FILES:
                        raise InputAdmissionError("file-count", child)
            require_same(before, os.fstat(fd), path)

        visit(self.root_fd, self.root, 0)
        return sorted(paths)


class SnapshotStream(io.BytesIO):
    """Present immutable memory at the upstream loader's original first read."""

    def __init__(self, store: SnapshotStore, name: str) -> None:
        super().__init__()
        self.store, self.name = store, name
        self.loaded = False

    def read(self, size: SupportsIndex | None = -1) -> bytes:
        if not self.loaded:
            data = self.store.read(self.name).data
            self.write(data)
            self.seek(0)
            self.loaded = True
        return super().read(None if size is None else index(size))


class EntryLoader(SchemaLoader):
    """Retain upstream schema checking while replacing its two input hooks."""

    def __init__(self, store: SnapshotStore, schemafile: str) -> None:
        super().__init__(schemafile, disable_cache=True)
        self.store = store
        self.entry = store.canonical_path(schemafile)
        self.parsed_entry: dict[str, Any] | None = None

    def get_schema_retrieval_uri(self) -> str:
        return self.entry.as_uri()

    def get_schema(self) -> dict[str, Any]:
        if self.parsed_entry is None:
            snapshot = self.store.read(self.entry)
            try:
                data = ParserSet().parse_data_with_path(
                    snapshot.data, self.entry, default_filetype="json", force_filetype="json"
                )
            except ValueError as error:
                raise SchemaParseError(str(self.entry)) from error
            if not isinstance(data, dict):
                raise SchemaParseError(str(self.entry))
            self.parsed_entry = cast(dict[str, Any], data)
        return self.parsed_entry


class LocalRegistryLoader(SchemaLoaderBase):
    """Lazily compose the effective upstream validator with local resources."""

    def __init__(self, store: SnapshotStore, schemafile: str) -> None:
        self.store = store
        self.entry_loader = EntryLoader(store, schemafile)
        self.aliases: dict[str, Snapshot] = {}
        self.validator: Validator | None = None
        self.preload_error: Exception | None = None
        self.parsers = ParserSet()

    def register_alias(self, alias: str, snapshot: Snapshot) -> None:
        previous = self.aliases.get(alias)
        if previous is not None and previous.path != snapshot.path:
            raise InputAdmissionError("alias-collision", alias)
        self.aliases[alias] = snapshot

    def resource(self, snapshot: Snapshot) -> Resource[Any]:
        if snapshot.path == self.entry_loader.entry:
            data: Any = self.entry_loader.get_schema()
        else:
            data = self.parsers.parse_data_with_path(
                snapshot.data, snapshot.path, default_filetype="json", force_filetype="json"
            )
        return Resource.from_contents(data, default_specification=DRAFT202012)

    def retrieve(self, uri: str) -> Resource[Any]:
        snapshot = self.aliases.get(uri)
        if snapshot is None:
            raise NoSuchResource(ref=uri)
        return self.resource(snapshot)

    def prepare_registry(self) -> Registry[Any]:
        paths = sorted(set(self.store.schema_paths()) | {self.entry_loader.entry})
        for path in paths:
            snapshot = self.store.read(path)
            self.register_alias(path.as_uri(), snapshot)
            relative = os.path.relpath(path, self.entry_loader.entry.parent)
            self.register_alias(quote(Path(relative).as_posix(), safe="/"), snapshot)
        registry: Registry[Any] = Registry(retrieve=self.retrieve)
        try:
            unique = {snapshot.path: snapshot for snapshot in self.aliases.values()}
            resources = {path: self.resource(snapshot) for path, snapshot in unique.items()}
            return registry.with_resources(
                (alias, resources[snapshot.path]) for alias, snapshot in self.aliases.items()
            ).crawl()
        except InputAdmissionError, MemoryError, TimeoutError:
            raise
        except Exception as error:
            self.preload_error = error
            return Registry(retrieve=self.retrieve)

    def get_validator(
        self,
        path: Path | str,
        instance_doc: dict[str, Any],
        format_opts: FormatOptions,
        regex_impl: RegexImplementation,
        fill_defaults: bool,
    ) -> Validator:
        if self.validator is None:
            template = cast(
                "_ConfiguredValidator",
                self.entry_loader.get_validator(
                    path, instance_doc, format_opts, regex_impl, fill_defaults
                ),
            )
            registry = self.prepare_registry()
            self.validator = type(template)(
                template.schema, registry=registry, format_checker=template.format_checker
            )
        return self.validator


@click.command(help="Validate explicit JSON instances against local contract schemas.")
@click.help_option("-h", "--help")
@click.option("--schemafile", multiple=True, required=True, metavar="PATH")
@click.option("--force-filetype", type=click.Choice(["json"]), default="json")
@click.option("--regex-variant", type=click.Choice(["default"]), default="default")
@click.option("--no-cache", is_flag=True)
@click.option("-o", "--output-format", type=click.Choice(["text", "json"]), default="text")
@click.option("--traceback-mode", type=click.Choice(["short", "full"]), default="short")
@click.option("-v", "--verbose", count=True)
@click.option("-q", "--quiet", count=True)
@click.argument("instancefiles", nargs=-1, required=True)
def main(
    schemafile: tuple[str, ...],
    force_filetype: str,
    regex_variant: str,
    no_cache: bool,
    output_format: str,
    traceback_mode: Literal["short", "full"],
    verbose: int,
    quiet: int,
    instancefiles: tuple[str, ...],
) -> None:
    if len(schemafile) != 1 or schemafile[0] == "-":
        raise click.UsageError("Provide exactly one local --schemafile path.")
    location = urlsplit(schemafile[0])
    if location.scheme or location.netloc:
        raise click.UsageError("Provide exactly one local --schemafile path.")
    if not no_cache or force_filetype != "json" or regex_variant != "default":
        raise click.UsageError("Use --force-filetype json --regex-variant default --no-cache.")
    if any(name == "-" or urlsplit(name).scheme for name in instancefiles):
        raise click.UsageError("Provide explicit local JSON instance paths.")
    with SnapshotStore(Path.cwd() / "contracts") as store:
        for name in instancefiles:
            store.preflight(name)
        streams = [SnapshotStream(store, name) for name in instancefiles]
        try:
            regex = RegexImplementation(RegexVariantName.default)
            checker = SchemaChecker(
                LocalRegistryLoader(store, schemafile[0]),
                InstanceLoader(streams, default_filetype="json", force_filetype="json"),
                REPORTER_BY_NAME[output_format](verbosity=1 + verbose - quiet),
                format_opts=FormatOptions(regex_impl=regex),
                regex_impl=regex,
                traceback_mode=traceback_mode,
            )
            result = checker.run()
        finally:
            for stream in streams:
                stream.close()
    click.get_current_context().exit(result)


if __name__ == "__main__":
    main()
