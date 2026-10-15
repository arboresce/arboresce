"""Independent whole-source admission and immutable bounded JCS selections.

This reference consumer does not apply the unsigned control/export JSON subset
to source values. Only checked private values reach the pinned JCS writer.
"""

from __future__ import annotations

import hashlib
import io
import json
import math
import re
from dataclasses import dataclass, field
from enum import Enum
from typing import IO, Literal, NoReturn, Self, cast, final

import rfc8785

MAX_INPUT_BYTES = 10_485_760
MAX_WHOLE_DEPTH = 128
MAX_WHOLE_NODES = 262_144
MAX_TOTAL_MEMBERS = 131_072
MAX_OBJECT_MEMBERS = 65_536
MAX_ARRAY_ELEMENTS = 131_072
MAX_NUMBER_BYTES = 4096
MAX_SELECTED_DEPTH = 64
MAX_SELECTED_NODES = 65_536
MAX_SELECTED_BYTES = 10_485_760
MAX_POINTER_SCALARS = 4096
MAX_POINTER_TOKENS = 64

_NUMBER = re.compile(r"-?(0|[1-9][0-9]*)(?:\.([0-9]+))?(?:[eE]([+-]?)([0-9]+))?")
_INDEX = re.compile(r"0|[1-9][0-9]*")
type _Scalar = None | bool | float | str
type _Tree = _Scalar | list[_Tree] | dict[str, _Tree]
type _Kind = Literal["null", "bool", "number", "string", "array", "object"]


class SourceFailure(Enum):
    """Static rejection stages; no input, exception traceback or partial output."""

    PARSE = ("parse", "invalid source JSON", "InvalidSourceJson")
    POINTER = ("pointer", "invalid JSON pointer", "InvalidJsonPointer")
    SELECT = ("select", "invalid JSON selection", "InvalidJsonSelection")

    @property
    def stage(self) -> str:
        return self.value[0]

    def __str__(self) -> str:
        return self.value[1]

    def __repr__(self) -> str:
        return self.value[2]


class _Reject(Exception):
    """Private unit control flow; never returned through the processing API."""


@dataclass(frozen=True, slots=True)
class _Member:
    key: str
    sort_key: bytes
    child: int


@dataclass(frozen=True, slots=True)
class _Node:
    kind: _Kind
    scalar: _Scalar
    children: tuple[int, ...]
    members: tuple[_Member, ...]
    depth: int
    count: int
    size: int


@final
class SourceJson:
    """A completely checked source with flat immutable child ownership."""

    __slots__ = ("_nodes", "_root")
    _nodes: tuple[_Node, ...]
    _root: int

    def __init__(self) -> None:
        raise TypeError("source values require complete admission")

    def __setattr__(self, _name: str, _value: object) -> NoReturn:
        raise AttributeError("immutable source value")

    def __delattr__(self, _name: str) -> NoReturn:
        raise AttributeError("immutable source value")

    @staticmethod
    def parse(data: object) -> SourceJson | SourceFailure:
        """Validate exact immutable UTF-8 bytes, including all off-path values."""
        if type(data) is not bytes or len(data) > MAX_INPUT_BYTES:
            return SourceFailure.PARSE
        try:
            parser = _Parser(data.decode("utf-8"))
            root = parser.value(1)
            parser.whitespace()
            if parser.position != len(parser.text):
                raise _Reject
            source = object.__new__(SourceJson)
            object.__setattr__(source, "_nodes", tuple(parser.nodes))
            object.__setattr__(source, "_root", root)
            return source
        except _Reject, UnicodeError, json.JSONDecodeError, rfc8785.CanonicalizationError:
            return SourceFailure.PARSE

    def select(self, pointer: object) -> CanonicalJsonSelection | SourceFailure:
        """Resolve one exact pointer, then emit and hash a complete bounded value."""
        try:
            tokens = _pointer(pointer)
        except _Reject:
            return SourceFailure.POINTER
        try:
            index = self._root
            for token in tokens:
                node = self._nodes[index]
                if node.kind == "object":
                    for member in node.members:
                        if member.key == token:
                            index = member.child
                            break
                    else:
                        raise _Reject
                elif node.kind == "array":
                    if len(token) > 6 or _INDEX.fullmatch(token) is None:
                        raise _Reject
                    offset = int(token)
                    if offset >= len(node.children):
                        raise _Reject
                    index = node.children[offset]
                else:
                    raise _Reject
            selected = self._nodes[index]
            if (
                selected.depth > MAX_SELECTED_DEPTH
                or selected.count > MAX_SELECTED_NODES
                or selected.size > MAX_SELECTED_BYTES
            ):
                raise _Reject
            tree = _tree(self._nodes, index)
            sink = _Sink(selected.size)
            # The pinned writer consumes only write(bytes); no caller sink enters.
            rfc8785.dump(tree, cast("IO[bytes]", sink))
            canonical = sink.finish()
            result = object.__new__(CanonicalJsonSelection)
            object.__setattr__(result, "_canonical", canonical)
            object.__setattr__(result, "_digest", hashlib.sha256(canonical).digest())
            return result
        except _Reject, rfc8785.CanonicalizationError:
            return SourceFailure.SELECT

    def __repr__(self) -> str:
        return "SourceJson(<redacted>)"


@final
class CanonicalJsonSelection:
    """Immutable complete canonical bytes and their complete SHA-256 digest."""

    __slots__ = ("_canonical", "_digest")
    _canonical: bytes
    _digest: bytes

    def __init__(self) -> None:
        raise TypeError("selection values require complete admission")

    def __setattr__(self, _name: str, _value: object) -> NoReturn:
        raise AttributeError("immutable selection value")

    def __delattr__(self, _name: str) -> NoReturn:
        raise AttributeError("immutable selection value")

    def __eq__(self, other: object) -> bool:
        return (
            type(other) is CanonicalJsonSelection
            and self._canonical == other._canonical
            and self._digest == other._digest
        )

    def __hash__(self) -> int:
        return hash((self._canonical, self._digest))

    def as_bytes(self) -> bytes:
        return self._canonical

    def byte_length(self) -> int:
        return len(self._canonical)

    def sha256(self) -> bytes:
        return self._digest

    def __copy__(self) -> Self:
        return self

    def __deepcopy__(self, _memo: dict[int, object]) -> Self:
        return self

    def __repr__(self) -> str:
        return "CanonicalJsonSelection(<redacted>)"


def _decimal(token: str) -> tuple[bool, str, int]:
    if not token or len(token) > MAX_NUMBER_BYTES:
        raise _Reject
    match = _NUMBER.fullmatch(token)
    if match is None:
        raise _Reject
    fraction = match[2] or ""
    digits = (match[1] + fraction).lstrip("0")
    # Full grammar precedes zero; no arbitrarily long exponent reaches int().
    if not digits:
        return False, "0", 0
    coefficient = digits.rstrip("0")
    if len(coefficient) > 17:
        raise _Reject
    exponent_text = (match[4] or "").lstrip("0")
    if len(exponent_text) > 4:
        raise _Reject
    exponent = int(exponent_text or "0")
    if exponent > 8192:
        raise _Reject
    if match[3] == "-":
        exponent = -exponent
    return token[0] == "-", coefficient, exponent - len(fraction) + len(digits) - len(coefficient)


def _number_from_token(token: str) -> tuple[float, bytes]:
    original = _decimal(token)
    negative, coefficient, exponent = original
    value = float(f"{'-' if negative else ''}{coefficient}e{exponent}")
    if (
        not math.isfinite(value)
        or abs(value) > 9_007_199_254_740_991
        or (value == 0 and coefficient != "0")
    ):
        raise _Reject
    # Only an admitted scalar float enters this independently pinned formatter.
    canonical = rfc8785.dumps(value)
    if len(canonical) > 32 or _decimal(canonical.decode("ascii")) != original:
        raise _Reject
    return value, canonical


def _string_bytes(text: str) -> int:
    size = 2
    for char in text:
        code = ord(char)
        if char in '"\\\b\t\n\f\r':
            size += 2
        elif code < 32:
            size += 6
        elif code < 128:
            size += 1
        elif code < 2048:
            size += 2
        elif code < 65536:
            size += 3
        else:
            size += 4
    return size


def _ordered(members: list[_Member]) -> tuple[_Member, ...]:
    count = len(members)
    if count < 2:
        return tuple(members)
    current, scratch = members, [members[0]] * count
    width = 1
    while width < count:
        for start in range(0, count, 2 * width):
            middle, end = min(start + width, count), min(start + 2 * width, count)
            left, right = start, middle
            for slot in range(start, end):
                if left < middle and (
                    right == end or current[left].sort_key <= current[right].sort_key
                ):
                    scratch[slot] = current[left]
                    left += 1
                else:
                    scratch[slot] = current[right]
                    right += 1
        current, scratch = scratch, current
        width *= 2
    for offset in range(1, count):
        if current[offset - 1].sort_key == current[offset].sort_key:
            raise _Reject
    return tuple(current)


@dataclass(slots=True)
class _Parser:
    text: str
    position: int = 0
    count: int = 0
    total_members: int = 0
    nodes: list[_Node] = field(default_factory=list[_Node])

    def whitespace(self) -> None:
        while self.position < len(self.text) and self.text[self.position] in " \t\r\n":
            self.position += 1

    def take(self, char: str) -> bool:
        if self.position < len(self.text) and self.text[self.position] == char:
            self.position += 1
            return True
        return False

    def value(self, depth: int) -> int:
        if depth > MAX_WHOLE_DEPTH or self.count == MAX_WHOLE_NODES:
            raise _Reject
        self.count += 1
        self.whitespace()
        if self.position == len(self.text):
            raise _Reject
        char = self.text[self.position]
        scalar: _Scalar = None
        children: tuple[int, ...] = ()
        members: tuple[_Member, ...] = ()
        if char == '"':
            kind: _Kind = "string"
            scalar = self.string()
            size = _string_bytes(scalar)
        elif char in "ntf":
            kind = "null" if char == "n" else "bool"
            literal, scalar = {"n": ("null", None), "t": ("true", True), "f": ("false", False)}[
                char
            ]
            if not self.text.startswith(literal, self.position):
                raise _Reject
            self.position += len(literal)
            size = len(literal)
        elif char == "[":
            kind = "array"
            children = self.array(depth)
            size = 2 + max(0, len(children) - 1)
        elif char == "{":
            kind = "object"
            members = self.object(depth)
            size = 2 + max(0, len(members) - 1)
            size += sum(_string_bytes(member.key) + 1 for member in members)
        elif char in "-0123456789":
            kind = "number"
            start = self.position
            while self.position < len(self.text) and self.text[self.position] in "-+.eE0123456789":
                if self.position - start == MAX_NUMBER_BYTES:
                    raise _Reject
                self.position += 1
            scalar, canonical = _number_from_token(self.text[start : self.position])
            size = len(canonical)
        else:
            raise _Reject
        subtree_depth, subtree_count = 1, 1
        for child in children:
            node = self.nodes[child]
            subtree_depth = max(subtree_depth, node.depth + 1)
            subtree_count += node.count
            size += node.size
        for member in members:
            node = self.nodes[member.child]
            subtree_depth = max(subtree_depth, node.depth + 1)
            subtree_count += node.count
            size += node.size
        index = len(self.nodes)
        self.nodes.append(
            _Node(kind, scalar, children, members, subtree_depth, subtree_count, size)
        )
        return index

    def string(self) -> str:
        if not self.take('"'):
            raise _Reject
        start = self.position - 1
        while self.position < len(self.text):
            char = self.text[self.position]
            self.position += 1
            if char == '"':
                # One already-delimited token, never an unbounded generic tree.
                value = cast("str", json.loads(self.text[start : self.position]))
                value.encode("utf-8")  # Reject unpaired Python surrogates before storage.
                return value
            if char == "\\":
                if self.position == len(self.text):
                    raise _Reject
                self.position += 1
            elif ord(char) < 32:
                raise _Reject
        raise _Reject

    def array(self, depth: int) -> tuple[int, ...]:
        self.position += 1
        self.whitespace()
        children: list[int] = []
        if self.take("]"):
            return ()
        while True:
            if len(children) == MAX_ARRAY_ELEMENTS:
                raise _Reject
            children.append(self.value(depth + 1))
            self.whitespace()
            if self.take("]"):
                return tuple(children)
            if not self.take(","):
                raise _Reject

    def object(self, depth: int) -> tuple[_Member, ...]:
        self.position += 1
        self.whitespace()
        members: list[_Member] = []
        if self.take("}"):
            return ()
        while True:
            if len(members) == MAX_OBJECT_MEMBERS or self.total_members == MAX_TOTAL_MEMBERS:
                raise _Reject
            self.total_members += 1
            self.whitespace()
            key = self.string()
            self.whitespace()
            if not self.take(":"):
                raise _Reject
            child = self.value(depth + 1)
            members.append(_Member(key, key.encode("utf-16be"), child))
            self.whitespace()
            if self.take("}"):
                return _ordered(members)
            if not self.take(","):
                raise _Reject


def _pointer(pointer: object) -> tuple[str, ...]:
    if type(pointer) is not str or len(pointer) > MAX_POINTER_SCALARS:
        raise _Reject
    if any(0xD800 <= ord(char) <= 0xDFFF for char in pointer):
        raise _Reject
    if not pointer:
        return ()
    if not pointer.startswith("/") or pointer.count("/") > MAX_POINTER_TOKENS:
        raise _Reject
    tokens = pointer[1:].split("/")
    for offset, token in enumerate(tokens):
        position = 0
        while position < len(token):
            if token[position] == "~":
                if position + 1 == len(token) or token[position + 1] not in "01":
                    raise _Reject
                position += 1
            position += 1
        tokens[offset] = token.replace("~1", "/").replace("~0", "~")
    return tuple(tokens)


@final
class _CanonicalKey(str):
    """Private serialization capsule; equality and ordinal hashes use identity."""

    __slots__ = ("_ordinal",)
    _ordinal: int

    def __new__(cls, *_args: object, **_kwargs: object) -> NoReturn:
        raise TypeError("keys require checked private construction")

    def __hash__(self) -> int:
        return self._ordinal

    def __eq__(self, other: object) -> bool:
        return self is other

    def __ne__(self, other: object) -> bool:
        return self is not other

    def __setattr__(self, _name: str, _value: object) -> NoReturn:
        raise AttributeError("immutable private key")

    def __delattr__(self, _name: str) -> NoReturn:
        raise AttributeError("immutable private key")

    def __init_subclass__(cls, **_kwargs: object) -> NoReturn:
        raise TypeError("private key subclasses are unsupported")


def _key(text: str, ordinal: int) -> _CanonicalKey:
    key = str.__new__(_CanonicalKey, text)
    object.__setattr__(key, "_ordinal", ordinal)
    return key


def _tree(nodes: tuple[_Node, ...], index: int) -> _Tree:
    node = nodes[index]
    if node.kind == "array":
        return [_tree(nodes, child) for child in node.children]
    if node.kind == "object":
        output: dict[str, _Tree] = {}
        for ordinal, member in enumerate(node.members):
            output[_key(member.key, ordinal)] = _tree(nodes, member.child)
        return output
    return node.scalar


class _Sink:
    __slots__ = ("_buffer", "_limit", "_size")

    def __init__(self, limit: int) -> None:
        self._buffer = io.BytesIO()
        self._limit, self._size = limit, 0

    def write(self, data: bytes) -> int:
        if len(data) > self._limit - self._size:
            raise _Reject
        self._buffer.write(data)
        self._size += len(data)
        return len(data)

    def finish(self) -> bytes:
        if self._size != self._limit:
            raise _Reject
        return self._buffer.getvalue()
