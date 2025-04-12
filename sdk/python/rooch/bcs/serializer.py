#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import struct
from enum import Enum
from typing import Any, Dict, List, Optional, Tuple, Type, TypeVar, Union

from ..utils.bytes import to_bytes
from ..utils.hex import to_hex


class BcsSerializationError(Exception):
    """Error when serializing data to BCS format"""
    pass


class BcsDeserializationError(Exception):
    """Error when deserializing data from BCS format"""
    pass


class BcsSerializer:
    """Binary Canonical Serialization for Rooch transactions"""
    
    @staticmethod
    def serialize_u8(value: int) -> bytes:
        """Serialize a u8 value
        
        Args:
            value: Integer value (0-255)
            
        Returns:
            Serialized bytes
            
        Raises:
            BcsSerializationError: If the value is out of range
        """
        if not isinstance(value, int) or value < 0 or value > 255:
            raise BcsSerializationError(f"Invalid u8 value: {value}")
        
        return struct.pack("<B", value)
    
    @staticmethod
    def serialize_u16(value: int) -> bytes:
        """Serialize a u16 value
        
        Args:
            value: Integer value (0-65535)
            
        Returns:
            Serialized bytes
            
        Raises:
            BcsSerializationError: If the value is out of range
        """
        if not isinstance(value, int) or value < 0 or value > 65535:
            raise BcsSerializationError(f"Invalid u16 value: {value}")
        
        return struct.pack("<H", value)
    
    @staticmethod
    def serialize_u32(value: int) -> bytes:
        """Serialize a u32 value
        
        Args:
            value: Integer value (0-4294967295)
            
        Returns:
            Serialized bytes
            
        Raises:
            BcsSerializationError: If the value is out of range
        """
        if not isinstance(value, int) or value < 0 or value > 4294967295:
            raise BcsSerializationError(f"Invalid u32 value: {value}")
        
        return struct.pack("<I", value)
    
    @staticmethod
    def serialize_u64(value: int) -> bytes:
        """Serialize a u64 value
        
        Args:
            value: Integer value (0-18446744073709551615)
            
        Returns:
            Serialized bytes
            
        Raises:
            BcsSerializationError: If the value is out of range
        """
        if not isinstance(value, int) or value < 0 or value > 18446744073709551615:
            raise BcsSerializationError(f"Invalid u64 value: {value}")
        
        return struct.pack("<Q", value)
    
    @staticmethod
    def serialize_u128(value: int) -> bytes:
        """Serialize a u128 value
        
        Args:
            value: Integer value (0-340282366920938463463374607431768211455)
            
        Returns:
            Serialized bytes
            
        Raises:
            BcsSerializationError: If the value is out of range
        """
        if not isinstance(value, int) or value < 0 or value > 340282366920938463463374607431768211455:
            raise BcsSerializationError(f"Invalid u128 value: {value}")
        
        # Split into two u64 values (low and high bits)
        low_bits = value & 0xFFFFFFFFFFFFFFFF
        high_bits = value >> 64
        
        return struct.pack("<QQ", low_bits, high_bits)
    
    @staticmethod
    def serialize_u256(value: int) -> bytes:
        """Serialize a u256 value
        
        Args:
            value: Integer value (0-2^256-1)
            
        Returns:
            Serialized bytes
            
        Raises:
            BcsSerializationError: If the value is out of range
        """
        if not isinstance(value, int) or value < 0 or value > (2**256 - 1):
            raise BcsSerializationError(f"Invalid u256 value: {value}")
        
        # Split into four u64 values
        part1 = value & 0xFFFFFFFFFFFFFFFF
        part2 = (value >> 64) & 0xFFFFFFFFFFFFFFFF
        part3 = (value >> 128) & 0xFFFFFFFFFFFFFFFF
        part4 = (value >> 192) & 0xFFFFFFFFFFFFFFFF
        
        return struct.pack("<QQQQ", part1, part2, part3, part4)
    
    @staticmethod
    def serialize_bool(value: bool) -> bytes:
        """Serialize a boolean value
        
        Args:
            value: Boolean value
            
        Returns:
            Serialized bytes
        """
        return struct.pack("<B", 1 if value else 0)
    
    @staticmethod
    def serialize_string(value: str) -> bytes:
        """Serialize a string
        
        Args:
            value: String value
            
        Returns:
            Serialized bytes
        """
        utf8_bytes = value.encode("utf-8")
        length = len(utf8_bytes)
        return BcsSerializer.serialize_len(length) + utf8_bytes
    
    @staticmethod
    def serialize_bytes(value: Union[bytes, bytearray]) -> bytes:
        """Serialize a bytes object
        
        Args:
            value: Bytes or bytearray
            
        Returns:
            Serialized bytes
        """
        return BcsSerializer.serialize_len(len(value)) + bytes(value)
    
    @staticmethod
    def serialize_address(address: str) -> bytes:
        """Serialize a Rooch address (32 bytes)
        
        Args:
            address: Rooch address string
            
        Returns:
            Serialized bytes (32 bytes)
            
        Raises:
            ValueError: If the address is invalid
            BcsSerializationError: If serialization fails
        """
        try:
            # Import here to avoid circular dependency if Address uses Serializer
            from ..address.rooch import RoochAddress 
            return RoochAddress(address).to_bytes()
        except ValueError as e:
            raise BcsSerializationError(f"Invalid address for BCS serialization: {str(e)}")
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize address: {str(e)}")
    
    @staticmethod
    def serialize_len(value: int) -> bytes:
        """Serialize a length (ULEB128 encoding)
        
        Args:
            value: Length value
            
        Returns:
            Serialized bytes
        """
        if value < 0:
            raise BcsSerializationError("Length cannot be negative")
        
        result = []
        while True:
            byte = value & 0x7F
            value >>= 7
            if value == 0:
                result.append(byte)
                break
            else:
                result.append(byte | 0x80)
        
        return bytes(result)
    
    @staticmethod
    def serialize_fixed_bytes(value: Union[bytes, bytearray, str]) -> bytes:
        """Serialize fixed-length bytes
        
        Args:
            value: Bytes, bytearray, or hex string
            
        Returns:
            Serialized bytes
            
        Raises:
            BcsSerializationError: If the value is invalid
        """
        try:
            if isinstance(value, str):
                # Try to interpret as hex string
                from ..utils.hex import from_hex
                return from_hex(value)
            else:
                return bytes(value)
        except Exception as e:
            raise BcsSerializationError(f"Invalid fixed bytes: {str(e)}")
    
    @staticmethod
    def serialize_vector(value: List[Any], item_serializer) -> bytes:
        """Serialize a vector/list of items
        
        Args:
            value: List of items
            item_serializer: Function to serialize each item
            
        Returns:
            Serialized bytes
        """
        result = BcsSerializer.serialize_len(len(value))
        for item in value:
            result += item_serializer(item)
        return result
    
    @staticmethod
    def serialize_option(value: Optional[Any], item_serializer) -> bytes:
        """Serialize an optional value
        
        Args:
            value: Optional value (None or some value)
            item_serializer: Function to serialize the value if present
            
        Returns:
            Serialized bytes
        """
        if value is None:
            return BcsSerializer.serialize_u8(0)
        else:
            return BcsSerializer.serialize_u8(1) + item_serializer(value)
    
    @staticmethod
    def serialize_map(value: Dict[Any, Any], key_serializer, value_serializer) -> bytes:
        """Serialize a map/dictionary
        
        Args:
            value: Dictionary
            key_serializer: Function to serialize keys
            value_serializer: Function to serialize values
            
        Returns:
            Serialized bytes
        """
        result = BcsSerializer.serialize_len(len(value))
        for k, v in value.items():
            result += key_serializer(k)
            result += value_serializer(v)
        return result

    @staticmethod
    def serialize_struct_tag(tag: 'StructTag') -> bytes:
        """Serialize a StructTag

        Args:
            tag: StructTag object

        Returns:
            Serialized bytes

        Raises:
            BcsSerializationError: If serialization fails
        """
        # Assumes StructTag type is imported or available
        from ..address.rooch import RoochAddress # Correct import needed
        from ..transactions.types import StructTag, TypeTag # For type hint checking
        
        if not isinstance(tag, StructTag):
             raise BcsSerializationError(f"Expected StructTag, got {type(tag)}")
             
        # Sequence: address, module_name, name, type_params
        try:
            # Assuming RoochAddress is the correct 32-byte address representation
            # Use from_hex_literal to handle short addresses
            addr = RoochAddress.from_hex_literal(tag.address)
            result = BcsSerializer.serialize_fixed_bytes(addr.to_bytes())
            result += BcsSerializer.serialize_string(tag.module)
            result += BcsSerializer.serialize_string(tag.name)
            result += BcsSerializer.serialize_vector(tag.type_params, BcsSerializer.serialize_type_tag)
            return result
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize StructTag: {e}") from e

    @staticmethod
    def serialize_type_tag(tag: 'TypeTag') -> bytes:
        """Serialize a TypeTag

        Args:
            tag: TypeTag object

        Returns:
            Serialized bytes

        Raises:
            BcsSerializationError: If serialization fails
        """
        # Assumes TypeTag, TypeTagCode types are imported or available
        from ..transactions.types import TypeTag, TypeTagCode, StructTag

        if not isinstance(tag, TypeTag):
            raise BcsSerializationError(f"Expected TypeTag, got {type(tag)}")
            
        # Serialize the variant index (enum value)
        # Use serialize_u8 as TypeTag codes fit in u8
        result = BcsSerializer.serialize_u8(tag.type_code.value)

        try:
            if tag.type_code == TypeTagCode.VECTOR:
                if not isinstance(tag.value, TypeTag):
                     raise BcsSerializationError("Vector TypeTag value must be another TypeTag")
                result += BcsSerializer.serialize_type_tag(tag.value)
            elif tag.type_code == TypeTagCode.STRUCT:
                if not isinstance(tag.value, StructTag):
                     raise BcsSerializationError("Struct TypeTag value must be a StructTag")
                result += BcsSerializer.serialize_struct_tag(tag.value)
            # Other types (bool, u8, u64, etc.) only have the index
            return result
        except Exception as e:
             raise BcsSerializationError(f"Failed to serialize TypeTag {tag.type_code.name}: {e}") from e

    @staticmethod
    def serialize_module_id(module_id: 'ModuleId') -> bytes:
        """Serialize a ModuleId

        Args:
            module_id: ModuleId object

        Returns:
            Serialized bytes
        """
        from ..address.rooch import RoochAddress
        from ..transactions.types import ModuleId
        
        if not isinstance(module_id, ModuleId):
             raise BcsSerializationError(f"Expected ModuleId, got {type(module_id)}")
        
        # Sequence: address, name
        try:
            # Use from_hex_literal to handle short addresses like 0x1, 0x2, 0x3
            addr = RoochAddress.from_hex_literal(module_id.address)
            result = BcsSerializer.serialize_fixed_bytes(addr.to_bytes())
            result += BcsSerializer.serialize_string(module_id.name)
            return result
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize ModuleId: {e}") from e

    @staticmethod
    def serialize_function_id(function_id: 'FunctionId') -> bytes:
        """Serialize a FunctionId

        Args:
            function_id: FunctionId object

        Returns:
            Serialized bytes
        """
        from ..transactions.types import FunctionId
        
        if not isinstance(function_id, FunctionId):
            raise BcsSerializationError(f"Expected FunctionId, got {type(function_id)}")
            
        # Sequence: module_id, function_name
        try:
            result = BcsSerializer.serialize_module_id(function_id.module_id)
            result += BcsSerializer.serialize_string(function_id.function_name)
            return result
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize FunctionId: {e}") from e

    @staticmethod
    def serialize_auth_payload(payload: 'AuthPayload') -> bytes:
        """Serialize an AuthPayload structure (for BitcoinAuthenticator)."""
        # Assumes AuthPayload type is imported
        from ..transactions.types import AuthPayload
        if not isinstance(payload, AuthPayload):
             raise BcsSerializationError(f"Expected AuthPayload, got {type(payload)}")
             
        # Sequence: signature, message_prefix, message_info, public_key, from_address
        try:
            result = BcsSerializer.serialize_bytes(payload.signature)
            result += BcsSerializer.serialize_bytes(payload.message_prefix)
            result += BcsSerializer.serialize_bytes(payload.message_info)
            result += BcsSerializer.serialize_bytes(payload.public_key)
            result += BcsSerializer.serialize_string(payload.from_address)
            return result
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize AuthPayload: {e}") from e


class BcsDeserializer:
    """Deserializer for BCS encoded data"""
    
    def __init__(self, data: bytes):
        """Initialize with BCS encoded data
        
        Args:
            data: BCS encoded data
        """
        self.data = data
        self.cursor = 0
    
    def deserialize_u8(self) -> int:
        """Deserialize a u8 value
        
        Returns:
            Deserialized u8 value
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        if self.cursor + 1 > len(self.data):
            raise BcsDeserializationError("Not enough data to deserialize u8")
        
        value = struct.unpack("<B", self.data[self.cursor:self.cursor + 1])[0]
        self.cursor += 1
        return value
    
    def deserialize_u16(self) -> int:
        """Deserialize a u16 value
        
        Returns:
            Deserialized u16 value
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        if self.cursor + 2 > len(self.data):
            raise BcsDeserializationError("Not enough data to deserialize u16")
        
        value = struct.unpack("<H", self.data[self.cursor:self.cursor + 2])[0]
        self.cursor += 2
        return value
    
    def deserialize_u32(self) -> int:
        """Deserialize a u32 value
        
        Returns:
            Deserialized u32 value
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        if self.cursor + 4 > len(self.data):
            raise BcsDeserializationError("Not enough data to deserialize u32")
        
        value = struct.unpack("<I", self.data[self.cursor:self.cursor + 4])[0]
        self.cursor += 4
        return value
    
    def deserialize_u64(self) -> int:
        """Deserialize a u64 value
        
        Returns:
            Deserialized u64 value
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        if self.cursor + 8 > len(self.data):
            raise BcsDeserializationError("Not enough data to deserialize u64")
        
        value = struct.unpack("<Q", self.data[self.cursor:self.cursor + 8])[0]
        self.cursor += 8
        return value
    
    def deserialize_u128(self) -> int:
        """Deserialize a u128 value
        
        Returns:
            Deserialized u128 value
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        if self.cursor + 16 > len(self.data):
            raise BcsDeserializationError("Not enough data to deserialize u128")
        
        low_bits, high_bits = struct.unpack("<QQ", self.data[self.cursor:self.cursor + 16])
        self.cursor += 16
        return (high_bits << 64) | low_bits
    
    def deserialize_u256(self) -> int:
        """Deserialize a u256 value
        
        Returns:
            Deserialized u256 value
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        if self.cursor + 32 > len(self.data):
            raise BcsDeserializationError("Not enough data to deserialize u256")
        
        part1, part2, part3, part4 = struct.unpack("<QQQQ", self.data[self.cursor:self.cursor + 32])
        self.cursor += 32
        return part1 | (part2 << 64) | (part3 << 128) | (part4 << 192)
    
    def deserialize_bool(self) -> bool:
        """Deserialize a boolean value
        
        Returns:
            Deserialized boolean value
            
        Raises:
            BcsDeserializationError: If there is not enough data or the value is invalid
        """
        value = self.deserialize_u8()
        if value > 1:
            raise BcsDeserializationError(f"Invalid boolean value: {value}")
        return value == 1
    
    def deserialize_len(self) -> int:
        """Deserialize a length (ULEB128 encoding)
        
        Returns:
            Deserialized length
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        result = 0
        shift = 0
        
        while True:
            if self.cursor >= len(self.data):
                raise BcsDeserializationError("Not enough data to deserialize length")
            
            byte = self.data[self.cursor]
            self.cursor += 1
            
            result |= ((byte & 0x7F) << shift)
            if byte & 0x80 == 0:
                break
            
            shift += 7
            if shift > 63:
                raise BcsDeserializationError("ULEB128 length overflow")
        
        return result
    
    def deserialize_string(self) -> str:
        """Deserialize a string
        
        Returns:
            Deserialized string
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        length = self.deserialize_len()
        if self.cursor + length > len(self.data):
            raise BcsDeserializationError("Not enough data to deserialize string")
        
        string_bytes = self.data[self.cursor:self.cursor + length]
        self.cursor += length
        
        try:
            return string_bytes.decode("utf-8")
        except UnicodeDecodeError as e:
            raise BcsDeserializationError(f"Invalid UTF-8 string: {str(e)}")
    
    def deserialize_bytes(self) -> bytes:
        """Deserialize a bytes object
        
        Returns:
            Deserialized bytes
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        length = self.deserialize_len()
        if self.cursor + length > len(self.data):
            raise BcsDeserializationError("Not enough data to deserialize bytes")
        
        result = self.data[self.cursor:self.cursor + length]
        self.cursor += length
        return result
    
    def deserialize_fixed_bytes(self, length: int) -> bytes:
        """Deserialize fixed-length bytes
        
        Args:
            length: Expected length of bytes
            
        Returns:
            Deserialized bytes
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        if self.cursor + length > len(self.data):
            raise BcsDeserializationError(f"Not enough data to deserialize {length} bytes")
        
        result = self.data[self.cursor:self.cursor + length]
        self.cursor += length
        return result
    
    def deserialize_vector(self, item_deserializer) -> List[Any]:
        """Deserialize a vector/list of items
        
        Args:
            item_deserializer: Function to deserialize each item
            
        Returns:
            Deserialized list
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        length = self.deserialize_len()
        result = []
        for _ in range(length):
            result.append(item_deserializer())
        return result
    
    def deserialize_option(self, item_deserializer) -> Optional[Any]:
        """Deserialize an optional value
        
        Args:
            item_deserializer: Function to deserialize the value if present
            
        Returns:
            Deserialized optional value
            
        Raises:
            BcsDeserializationError: If there is not enough data or the value is invalid
        """
        has_value = self.deserialize_bool()
        if has_value:
            return item_deserializer()
        else:
            return None
    
    def deserialize_map(self, key_deserializer, value_deserializer) -> Dict[Any, Any]:
        """Deserialize a map/dictionary
        
        Args:
            key_deserializer: Function to deserialize keys
            value_deserializer: Function to deserialize values
            
        Returns:
            Deserialized dictionary
            
        Raises:
            BcsDeserializationError: If there is not enough data
        """
        length = self.deserialize_len()
        result = {}
        for _ in range(length):
            key = key_deserializer()
            value = value_deserializer()
            result[key] = value
        return result


class Args:
    """Helper class for creating arguments for function calls"""
    
    @staticmethod
    def u8(value: int) -> List[int]:
        """Create a u8 argument
        
        Args:
            value: Integer value
            
        Returns:
            Serialized argument
        """
        return [0, value]
    
    @staticmethod
    def u16(value: int) -> List[Any]:
        """Create a u16 argument
        
        Args:
            value: Integer value
            
        Returns:
            Serialized argument
        """
        return [1, value]
    
    @staticmethod
    def u32(value: int) -> List[Any]:
        """Create a u32 argument
        
        Args:
            value: Integer value
            
        Returns:
            Serialized argument
        """
        return [2, value]
    
    @staticmethod
    def u64(value: int) -> List[Any]:
        """Create a u64 argument
        
        Args:
            value: Integer value
            
        Returns:
            Serialized argument
        """
        return [3, str(value)]
    
    @staticmethod
    def u128(value: int) -> List[Any]:
        """Create a u128 argument
        
        Args:
            value: Integer value
            
        Returns:
            Serialized argument
        """
        return [4, str(value)]
    
    @staticmethod
    def u256(value: int) -> List[Any]:
        """Create a u256 argument
        
        Args:
            value: Integer value
            
        Returns:
            Serialized argument
        """
        return [5, str(value)]
    
    @staticmethod
    def bool(value: bool) -> List[Any]:
        """Create a boolean argument
        
        Args:
            value: Boolean value
            
        Returns:
            Serialized argument
        """
        return [6, value]
    
    @staticmethod
    def address(value: str) -> List[Any]:
        """Create an address argument
        
        Args:
            value: Address as hex string
            
        Returns:
            Serialized argument
        """
        return [7, value]
    
    @staticmethod
    def string(value: str) -> List[Any]:
        """Create a string argument
        
        Args:
            value: String value
            
        Returns:
            Serialized argument
        """
        return [8, value]
    
    @staticmethod
    def vector(element_type: str, values: List[Any]) -> List[Any]:
        """Create a vector argument
        
        Args:
            element_type: Type of vector elements
            values: List of values
            
        Returns:
            Serialized argument
        """
        return [9, {"type": element_type, "value": values}]
    
    @staticmethod
    def vec(element_type: str, values: List[Any]) -> List[Any]:
        """Alias for vector()
        
        Args:
            element_type: Type of vector elements
            values: List of values
            
        Returns:
            Serialized argument
        """
        return Args.vector(element_type, values)
    
    @staticmethod
    def objectId(value: str) -> List[Any]:
        """Create an object ID argument
        
        Args:
            value: Object ID
            
        Returns:
            Serialized argument
        """
        return [10, value]