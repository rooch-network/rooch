#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from enum import IntEnum
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple, Union

from ...bcs.serializer import BcsSerializer, Serializable, BcsDeserializer, Deserializable, BcsSerializationError, BcsDeserializationError
from ...address.rooch import RoochAddress
from ...utils.hex import ensure_hex_prefix


class TypeTagCode(IntEnum):
    BOOL = 0
    U8 = 1
    U64 = 2
    U128 = 3
    ADDRESS = 4
    # SIGNER = 5 # Cannot be passed as type arg
    VECTOR = 6
    STRUCT = 7
    U16 = 8
    U32 = 9
    U256 = 10
    # Add other types if needed


@dataclass
class StructTag(Serializable, Deserializable):
    """Represents a Move struct type tag."""
    address: RoochAddress  # Changed from str to RoochAddress
    module: str
    name: str
    type_params: List['TypeTag']

    def __init__(self, address: Union[str, RoochAddress], module: str, name: str, type_params: List['TypeTag']):
        """
        Args:
            address: Address as RoochAddress or hex string
            module: Module name
            name: Struct name
            type_params: Type parameters
        """
        # Convert string address to RoochAddress if needed
        if isinstance(address, str):
            self.address = RoochAddress.from_hex_literal(ensure_hex_prefix(address))
        else:
            self.address = address
        self.module = module
        self.name = name
        self.type_params = type_params

    def __str__(self) -> str:
        """String representation of the struct tag."""
        if self.type_params:
            params = ", ".join(map(str, self.type_params))
            return f"{self.address.to_hex_literal()}::{self.module}::{self.name}<{params}>"
        return f"{self.address.to_hex_literal()}::{self.module}::{self.name}"

    def __eq__(self, other: Union[str, 'StructTag']) -> bool:
        """Compare with another StructTag or string."""
        if isinstance(other, str):
            # Parse string like "0x3::gas_coin::RGas"
            parts = other.split("::")
            if len(parts) != 3:
                return False
            addr_str, module, name = parts
            # Compare each part
            return (self.address.to_hex_literal() == addr_str and
                   self.module == module and
                   self.name == name)
        elif isinstance(other, StructTag):
            return (self.address == other.address and
                   self.module == other.module and
                   self.name == other.name and
                   self.type_params == other.type_params)
        return False

    # --- BCS Implementation ---
    def serialize(self, serializer: BcsSerializer):
        # Sequence: address, module_name, name, type_params
        serializer.struct(self.address)
        serializer.str(self.module)
        serializer.str(self.name)
        serializer.sequence(self.type_params, BcsSerializer.struct)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'StructTag':
        # Sequence: address, module_name, name, type_params
        addr = RoochAddress.deserialize(deserializer)
        module = deserializer.str()
        name = deserializer.str()
        type_params = deserializer.sequence(lambda d: TypeTag.deserialize(d))
        return StructTag(address=addr, module=module, name=name, type_params=type_params)


@dataclass
class TypeTag(Serializable, Deserializable):
    type_code: TypeTagCode
    # value holds inner type for Vector, or StructTag for Struct
    value: Optional[Union['TypeTag', StructTag]] = None

    # --- BCS Implementation ---
    def serialize(self, serializer: BcsSerializer):
        serializer.u8(self.type_code.value) # Serialize the variant index
        if self.type_code == TypeTagCode.VECTOR:
            if not isinstance(self.value, TypeTag):
                raise TypeError("Vector TypeTag value must be another TypeTag")
            serializer.struct(self.value) # Inner TypeTag is Serializable
        elif self.type_code == TypeTagCode.STRUCT:
            if not isinstance(self.value, StructTag):
                raise TypeError("Struct TypeTag value must be a StructTag")
            serializer.struct(self.value) # StructTag is Serializable
        # Other types only need the index, which was already written

    def __eq__(self, other: Union[str, 'TypeTag']) -> bool:
        """Compare with another TypeTag or string."""
        if isinstance(other, str):
            if self.type_code == TypeTagCode.STRUCT:
                return str(self.value) == other
            return str(self) == other
        elif isinstance(other, TypeTag):
            if self.type_code != other.type_code:
                return False
            if self.type_code in (TypeTagCode.VECTOR, TypeTagCode.STRUCT):
                return self.value == other.value
            return True
        return False

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'TypeTag':
        type_code_val = deserializer.u8()
        try:
            type_code = TypeTagCode(type_code_val)
        except ValueError:
            raise BcsDeserializationError(f"Invalid TypeTagCode value: {type_code_val}")

        value = None
        if type_code == TypeTagCode.VECTOR:
            value = TypeTag.deserialize(deserializer) # Deserialize inner TypeTag
        elif type_code == TypeTagCode.STRUCT:
            value = StructTag.deserialize(deserializer) # Deserialize inner StructTag
        
        return TypeTag(type_code=type_code, value=value)

    @classmethod
    def bool(cls): return cls(TypeTagCode.BOOL)
    @classmethod
    def u8(cls): return cls(TypeTagCode.U8)
    @classmethod
    def u16(cls): return cls(TypeTagCode.U16)
    @classmethod
    def u32(cls): return cls(TypeTagCode.U32)
    @classmethod
    def u64(cls): return cls(TypeTagCode.U64)
    @classmethod
    def u128(cls): return cls(TypeTagCode.U128)
    @classmethod
    def u256(cls): return cls(TypeTagCode.U256)
    @classmethod
    def address(cls): return cls(TypeTagCode.ADDRESS)
    @classmethod
    def vector(cls, element_type: 'TypeTag'): return cls(TypeTagCode.VECTOR, element_type)
    @classmethod
    def struct(cls, struct_tag: StructTag): return cls(TypeTagCode.STRUCT, struct_tag)

    def __str__(self) -> str:
        if self.type_code == TypeTagCode.VECTOR:
            return f"vector<{self.value}>"
        elif self.type_code == TypeTagCode.STRUCT:
            s_tag: StructTag = self.value
            params = ", ".join(map(str, s_tag.type_params))
            return f"{s_tag.address.to_hex_literal()}::{s_tag.module}::{s_tag.name}<{params}>"
        else:
            return self.type_code.name.lower() 