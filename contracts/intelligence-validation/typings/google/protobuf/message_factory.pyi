from .descriptor import Descriptor
from .message import Message

def GetMessageClass(descriptor: Descriptor) -> type[Message]: ...
