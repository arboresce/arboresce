from collections.abc import Mapping

class OneofDescriptor:
    name: str

class EnumValueDescriptor: ...

class EnumDescriptor:
    values_by_number: Mapping[int, EnumValueDescriptor]

class FieldDescriptor:
    TYPE_UINT32: int
    TYPE_UINT64: int
    TYPE_ENUM: int
    TYPE_MESSAGE: int
    TYPE_STRING: int
    TYPE_BYTES: int
    type: int
    is_repeated: bool
    containing_oneof: OneofDescriptor | None
    message_type: Descriptor | None
    enum_type: EnumDescriptor | None

class Descriptor:
    fields_by_number: Mapping[int, FieldDescriptor]

class ServiceDescriptor: ...

class FileDescriptor:
    package: str
    message_types_by_name: Mapping[str, Descriptor]
    enum_types_by_name: Mapping[str, EnumDescriptor]
    services_by_name: Mapping[str, ServiceDescriptor]
