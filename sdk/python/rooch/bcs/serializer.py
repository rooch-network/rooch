#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import io
import struct
from enum import Enum
# Use typing_extensions for Protocol if supporting older Python versions, else typing
from typing import Any, Callable, Dict, List, Optional, Tuple, Type, TypeVar, Union, Protocol, runtime_checkable

# Re-add imports that might be needed by specific serializers/deserializers later
# Keep them minimal for now
# from ..utils.bytes import to_bytes # Likely replaced by io operations
# from ..utils.hex import to_hex, from_hex # Keep for hex handling if needed

# Constants from Aptos SDK for range checks
MAX_U8 = 2**8 - 1
MAX_U16 = 2**16 - 1
MAX_U32 = 2**32 - 1
MAX_U64 = 2**64 - 1
MAX_U128 = 2**128 - 1
MAX_U256 = 2**256 - 1


class BcsSerializationError(Exception):
    """Error when serializing data to BCS format"""
    pass


class BcsDeserializationError(Exception):
    """Error when deserializing data from BCS format"""
    pass


# --- Forward Declarations & Protocols ---

# Forward declare the classes for type hints in protocols
class BcsSerializer: pass
class BcsDeserializer: pass

# Define Deserializable Protocol (similar to Aptos SDK)
DeserializableT = TypeVar('DeserializableT', bound='Deserializable')
@runtime_checkable
class Deserializable(Protocol):
    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> DeserializableT: ...

    @classmethod
    def from_bytes(cls: Type[DeserializableT], data: bytes) -> DeserializableT:
        """Convenience method to deserialize directly from bytes."""
        deserializer = BcsDeserializer(data)
        value = cls.deserialize(deserializer)
        if deserializer.remaining() > 0:
            raise BcsDeserializationError(f"{deserializer.remaining()} remaining bytes after deserializing {cls.__name__}")
        return value

# Define Serializable Protocol (similar to Aptos SDK)
@runtime_checkable
class Serializable(Protocol):
    def serialize(self, serializer: BcsSerializer): ...

    def to_bytes(self) -> bytes:
        """Convenience method to serialize directly to bytes."""
        serializer = BcsSerializer()
        self.serialize(serializer)
        return serializer.output()


# --- Serializer ---

class BcsSerializer:
    """BCS Serializer using io.BytesIO based on Aptos SDK implementation.
    Handles basic types and containers. Complex types should implement Serializable.
    """
    _output: io.BytesIO

    def __init__(self):
        self._output = io.BytesIO()

    def output(self) -> bytes:
        """Return the serialized bytes"""
        return self._output.getvalue()

    def _write_int(self, value: int, length: int):
        """Helper to write integer values"""
        try:
            self._output.write(value.to_bytes(length, "little", signed=False))
        except OverflowError as e:
            raise BcsSerializationError(f"Integer value {value} is out of range for {length*8}-bit unsigned integer") from e

    def u8(self, value: int):
        """Serialize a u8 value"""
        if not 0 <= value <= MAX_U8:
            raise BcsSerializationError(f"Value {value} out of range for u8")
        self._write_int(value, 1)

    def u16(self, value: int):
        """Serialize a u16 value"""
        if not 0 <= value <= MAX_U16:
            raise BcsSerializationError(f"Value {value} out of range for u16")
        self._write_int(value, 2)

    def u32(self, value: int):
        """Serialize a u32 value"""
        if not 0 <= value <= MAX_U32:
            raise BcsSerializationError(f"Value {value} out of range for u32")
        self._write_int(value, 4)

    def u64(self, value: int):
        """Serialize a u64 value"""
        if not 0 <= value <= MAX_U64:
            raise BcsSerializationError(f"Value {value} out of range for u64")
        self._write_int(value, 8)

    def u128(self, value: int):
        """Serialize a u128 value"""
        if not 0 <= value <= MAX_U128:
            raise BcsSerializationError(f"Value {value} out of range for u128")
        self._write_int(value, 16)

    def u256(self, value: int):
        """Serialize a u256 value"""
        if not 0 <= value <= MAX_U256:
            raise BcsSerializationError(f"Value {value} out of range for u256")
        self._write_int(value, 32)

    def bool(self, value: bool):
        """Serialize a boolean value"""
        self.u8(1 if value else 0)

    def uleb128(self, value: int):
        """Serialize a length/integer using ULEB128 encoding"""
        if value < 0:
            raise BcsSerializationError("ULEB128 value cannot be negative")
        if value > MAX_U32: # Standard practice to limit ULEB128 to U32 for lengths
            raise BcsSerializationError(f"Cannot encode {value} into uleb128 (commonly limited to U32 range)")

        while value >= 0x80:
            byte = (value & 0x7F) | 0x80
            self.u8(byte)
            value >>= 7
        self.u8(value & 0x7F)

    def bytes(self, value: Union[bytes, bytearray]):
        """Serialize a variable-length byte sequence (length-prefixed)"""
        value_bytes = bytes(value)
        self.uleb128(len(value_bytes))
        self._output.write(value_bytes)

    def fixed_bytes(self, value: Union[bytes, bytearray]):
        """Serialize a fixed-length byte sequence (no length prefix)"""
        self._output.write(bytes(value))

    def str(self, value: str):
        """Serialize a string (UTF-8 encoded, length-prefixed)"""
        utf8_bytes = value.encode("utf-8")
        self.bytes(utf8_bytes)

    def sequence(self, values: List[Any], item_serializer_method: Callable[[BcsSerializer, Any], None]):
        """Serialize a sequence (list/vector) of items.
        
        Args:
            values: The list of items to serialize.
            item_serializer_method: A method (e.g., BcsSerializer.u8, BcsSerializer.struct)
                                     or a function that takes (serializer, item) and serializes the item.
        """
        self.uleb128(len(values))
        for item in values:
            item_serializer_method(self, item)

    def option(self, value: Optional[Any], item_serializer_method: Callable[[BcsSerializer, Any], None]):
        """Serialize an optional value.

        Args:
            value: The optional value (None or the item).
            item_serializer_method: The method/function to serialize the item if present.
        """
        if value is None:
            self.u8(0) # Presence byte: 0 for None
        else:
            self.u8(1) # Presence byte: 1 for Some
            item_serializer_method(self, value)

    def map(self, data: Dict[Any, Any], key_serializer_method: Callable[[BcsSerializer, Any], None], value_serializer_method: Callable[[BcsSerializer, Any], None]):
        """Serialize a map/dictionary, ensuring keys are sorted by serialized bytes.

        Args:
            data: The dictionary to serialize.
            key_serializer_method: The method/function to serialize a key.
            value_serializer_method: The method/function to serialize a value.
        """
        if not isinstance(data, dict):
            raise BcsSerializationError(f"Expected dict for map serialization, got {type(data)}")

        encoded_items = []
        for k, v in data.items():
            # Serialize key and value separately to get their byte representations
            # We need temporary serializers to capture the bytes of each key/value
            key_ser = BcsSerializer()
            key_serializer_method(key_ser, k)
            key_bytes = key_ser.output()

            val_ser = BcsSerializer()
            value_serializer_method(val_ser, v)
            val_bytes = val_ser.output()

            encoded_items.append((key_bytes, val_bytes))

        # Sort items based on the serialized key bytes (lexicographical comparison)
        encoded_items.sort(key=lambda item: item[0])

        # Write the number of items
        self.uleb128(len(encoded_items))

        # Write the sorted key-value pairs (bytes directly)
        for k_bytes, v_bytes in encoded_items:
            self.fixed_bytes(k_bytes) # Write key bytes
            self.fixed_bytes(v_bytes) # Write value bytes

    def struct(self, value: Serializable):
        """Serialize a struct or object that implements the Serializable protocol."""
        if not isinstance(value, Serializable):
            raise BcsSerializationError(f"Value of type {type(value)} does not implement Serializable protocol")
        value.serialize(self)


# --- Deserializer ---

class BcsDeserializer:
    """BCS Deserializer using io.BytesIO based on Aptos SDK implementation.
    Handles basic types and containers. Complex types should implement Deserializable.
    """
    _input: io.BytesIO
    _length: int

    def __init__(self, data: bytes):
        """Initialize with BCS encoded data"""
        if not isinstance(data, bytes):
            raise TypeError("Input data must be bytes")
        self._input = io.BytesIO(data)
        self._length = len(data)

    def remaining(self) -> int:
        """Return the number of remaining bytes to read"""
        return self._length - self._input.tell()

    def _read(self, length: int) -> bytes:
        """Helper to read exact number of bytes"""
        if length < 0:
            raise BcsDeserializationError("Cannot read negative number of bytes")
        read_bytes = self._input.read(length)
        if len(read_bytes) < length:
            raise BcsDeserializationError(f"Not enough data to read {length} bytes. Wanted {length}, got {len(read_bytes)}. Remaining: {self.remaining()}")
        return read_bytes

    def _read_int(self, length: int) -> int:
        """Helper to read and convert bytes to integer"""
        data = self._read(length)
        return int.from_bytes(data, byteorder="little", signed=False)

    def u8(self) -> int:
        """Deserialize a u8 value"""
        return self._read_int(1)

    def u16(self) -> int:
        """Deserialize a u16 value"""
        return self._read_int(2)

    def u32(self) -> int:
        """Deserialize a u32 value"""
        return self._read_int(4)

    def u64(self) -> int:
        """Deserialize a u64 value"""
        return self._read_int(8)

    def u128(self) -> int:
        """Deserialize a u128 value"""
        return self._read_int(16)

    def u256(self) -> int:
        """Deserialize a u256 value"""
        return self._read_int(32)

    def bool(self) -> bool:
        """Deserialize a boolean value"""
        value = self.u8()
        if value == 0:
            return False
        elif value == 1:
            return True
        else:
            raise BcsDeserializationError(f"Invalid boolean value: {value}")

    def uleb128(self) -> int:
        """Deserialize a ULEB128 encoded integer"""
        result = 0
        shift = 0
        while True:
            byte = self.u8() # Reads one byte, handles EOF check
            result |= (byte & 0x7F) << shift
            if (byte & 0x80) == 0:
                break
            shift += 7
            if shift > 35: # Limit check (5 bytes * 7 bits) to prevent excessive reads/shifts
                raise BcsDeserializationError("ULEB128 value too large (potential overflow or exceeds 32-bit common use)")
        return result

    def bytes(self) -> bytes:
        """Deserialize a variable-length byte sequence (length-prefixed)"""
        length = self.uleb128()
        return self._read(length)

    def fixed_bytes(self, length: int) -> bytes:
        """Deserialize a fixed-length byte sequence (no length prefix)"""
        return self._read(length)

    def str(self) -> str:
        """Deserialize a string (UTF-8 encoded, length-prefixed)"""
        string_bytes = self.bytes()
        try:
            return string_bytes.decode("utf-8")
        except UnicodeDecodeError as e:
            raise BcsDeserializationError(f"Invalid UTF-8 string: {str(e)}") from e

    def sequence(self, item_deserializer_method: Callable[[BcsDeserializer], Any]) -> List[Any]:
        """Deserialize a sequence (list/vector) of items.

        Args:
            item_deserializer_method: A method (e.g., BcsDeserializer.u8, BcsDeserializer.struct)
                                      or a function that takes (deserializer) and returns the deserialized item.
        """
        length = self.uleb128()
        result = []
        for _ in range(length):
            result.append(item_deserializer_method(self))
        return result

    def option(self, item_deserializer_method: Callable[[BcsDeserializer], Any]) -> Optional[Any]:
        """Deserialize an optional value.

        Args:
            item_deserializer_method: The method/function to deserialize the item if present.
        """
        presence = self.u8()
        if presence == 0:
            return None
        elif presence == 1:
            return item_deserializer_method(self)
        else:
            raise BcsDeserializationError(f"Invalid option presence byte: {presence}")

    def map(self, key_deserializer_method: Callable[[BcsDeserializer], Any], value_deserializer_method: Callable[[BcsDeserializer], Any]) -> Dict[Any, Any]:
        """Deserialize a map/dictionary. Assumes keys were serialized sorted."""
        length = self.uleb128()
        result = {}
        for _ in range(length):
            # Keys and values are read in the order they appear (which should be sorted by key bytes)
            key = key_deserializer_method(self)
            value = value_deserializer_method(self)
            if key in result:
                # While Python dict handles overwrites, duplicate keys violate BCS canonical form.
                # You might want to raise an error here for stricter adherence.
                # raise BcsDeserializationError(f"Duplicate key found during map deserialization: {key}")
                pass # Default Python dict behavior is to overwrite.
            result[key] = value
        return result

    def struct(self, cls: Type[DeserializableT]) -> DeserializableT:
        """Deserialize a struct or object that implements the Deserializable protocol."""
        if not issubclass(cls, Deserializable):
            raise BcsDeserializationError(f"Class {cls.__name__} does not implement Deserializable protocol")
        # Delegate deserialization logic to the class itself
        return cls.deserialize(self)


# --- Helper functions (Less critical now, but can be useful) ---

def bcs_encode(value: Serializable) -> bytes:
    """Utility function to encode a Serializable object."""
    if not isinstance(value, Serializable):
        raise TypeError(f"Value of type {type(value)} is not Serializable")
    return value.to_bytes()

def bcs_decode(data: bytes, cls: Type[DeserializableT]) -> DeserializableT:
    """Utility function to decode data into a Deserializable object."""
    if not issubclass(cls, Deserializable):
        raise TypeError(f"Class {cls.__name__} is not Deserializable")
    return cls.from_bytes(data)

class Args:
    """Helper class for creating transaction arguments with proper BCS serialization"""
    
    @staticmethod
    def u8(value: int) -> int:
        """Create a u8 argument"""
        if not 0 <= value <= MAX_U8:
            raise BcsSerializationError(f"Value {value} out of range for u8")
        return value

    @staticmethod
    def u16(value: int) -> int:
        """Create a u16 argument"""
        if not 0 <= value <= MAX_U16:
            raise BcsSerializationError(f"Value {value} out of range for u16")
        return value

    @staticmethod
    def u32(value: int) -> int:
        """Create a u32 argument"""
        if not 0 <= value <= MAX_U32:
            raise BcsSerializationError(f"Value {value} out of range for u32")
        return value

    @staticmethod
    def u64(value: int) -> int:
        """Create a u64 argument"""
        if not 0 <= value <= MAX_U64:
            raise BcsSerializationError(f"Value {value} out of range for u64")
        return value

    @staticmethod
    def u128(value: int) -> int:
        """Create a u128 argument"""
        if not 0 <= value <= MAX_U128:
            raise BcsSerializationError(f"Value {value} out of range for u128")
        return value

    @staticmethod
    def u256(value: int) -> int:
        """Create a u256 argument"""
        if not 0 <= value <= MAX_U256:
            raise BcsSerializationError(f"Value {value} out of range for u256")
        return value

    @staticmethod
    def bool(value: bool) -> bool:
        """Create a boolean argument"""
        return bool(value)

    @staticmethod
    def address(value: str) -> str:
        """Create an address argument"""
        # Ensure the address has 0x prefix
        if not value.startswith("0x"):
            value = "0x" + value
        return value

    @staticmethod
    def vector_u8(value: Union[bytes, bytearray, List[int]]) -> bytes:
        """Create a vector<u8> argument"""
        if isinstance(value, (bytes, bytearray)):
            return bytes(value)
        elif isinstance(value, list):
            return bytes(value)
        else:
            raise BcsSerializationError(f"Cannot convert {type(value)} to vector<u8>")

    @staticmethod
    def string(value: str) -> str:
        """Create a string argument"""
        return str(value)