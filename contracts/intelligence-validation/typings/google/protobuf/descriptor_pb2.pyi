from collections.abc import Iterable

from .message import Message

class FileDescriptorProto(Message): ...

class FileDescriptorSet(Message):
    file: Iterable[FileDescriptorProto]
