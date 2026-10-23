"""Closed protobuf framing for the pinned intelligence descriptor.

The message class is trusted configuration from the committed descriptor, never
a caller-selected schema. This layer does not admit complete stage semantics.
"""

from enum import Enum

from google.protobuf.descriptor import Descriptor, FieldDescriptor
from google.protobuf.message import DecodeError, Message

MAX_FRAME_BYTES = 1_048_576
MAX_MESSAGE_DEPTH = 16


class WireFailure(Enum):
    """One static refusal without source bytes, exceptions or partial messages."""

    FRAME = "invalid intelligence frame"


class _Reject(Exception):
    """Private control flow; never returned to a caller."""


def _varint(data: memoryview, position: int, end: int) -> tuple[int, int]:
    value = 0
    shift = 0
    while position < end and shift <= 63:
        byte = data[position]
        position += 1
        value |= (byte & 127) << shift
        if byte < 128:
            if (shift and byte == 0) or value > 18_446_744_073_709_551_615:
                raise _Reject
            return value, position
        shift += 7
    raise _Reject


def _message(data: memoryview, descriptor: Descriptor, depth: int) -> None:
    if depth > MAX_MESSAGE_DEPTH:
        raise _Reject
    position, end = 0, len(data)
    seen: set[int] = set()
    oneofs: set[str] = set()
    while position < end:
        tag, position = _varint(data, position, end)
        number, wire = tag >> 3, tag & 7
        field = descriptor.fields_by_number.get(number)
        if field is None or (not field.is_repeated and number in seen):
            raise _Reject
        seen.add(number)
        if field.containing_oneof is not None:
            name = field.containing_oneof.name
            if name in oneofs:
                raise _Reject
            oneofs.add(name)
        if field.type in (
            FieldDescriptor.TYPE_UINT32,
            FieldDescriptor.TYPE_UINT64,
            FieldDescriptor.TYPE_ENUM,
        ):
            if wire != 0:
                raise _Reject
            value, position = _varint(data, position, end)
            if field.type == FieldDescriptor.TYPE_UINT32 and value > 4_294_967_295:
                raise _Reject
            if field.type == FieldDescriptor.TYPE_ENUM:
                enum = field.enum_type
                if enum is None or value == 0 or value not in enum.values_by_number:
                    raise _Reject
        else:
            if wire != 2:
                raise _Reject
            size, position = _varint(data, position, end)
            if size > end - position:
                raise _Reject
            payload = data[position : position + size]
            position += size
            if field.type == FieldDescriptor.TYPE_MESSAGE:
                nested = field.message_type
                if nested is None:
                    raise _Reject
                _message(payload, nested, depth + 1)
            elif field.type == FieldDescriptor.TYPE_STRING:
                bytes(payload).decode("utf-8")
            elif field.type != FieldDescriptor.TYPE_BYTES:
                raise _Reject


def decode_frame(message_type: type[Message], data: object) -> Message | WireFailure:
    """Check exact immutable bytes completely before the pinned runtime decodes.

    A successful result proves framing only. Its caller must enforce complete
    fixed codecs, selected profile limits and all trusted semantic joins.
    """
    if type(data) is not bytes or len(data) > MAX_FRAME_BYTES:
        return WireFailure.FRAME
    try:
        _message(memoryview(data), message_type.DESCRIPTOR, 1)
        result = message_type()
        result.ParseFromString(data)
        return result
    except _Reject, UnicodeError, DecodeError:
        return WireFailure.FRAME
