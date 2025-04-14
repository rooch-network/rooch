#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from dataclasses import dataclass
from typing import Union

from ...address.rooch import RoochAddress
from ...utils.hex import ensure_hex_prefix
from ...bcs.serializer import BcsSerializer, Serializable, BcsDeserializer, Deserializable


@dataclass
class ModuleId(Serializable, Deserializable):
    """Represents a Move module identifier."""
    address: RoochAddress
    name: str    # Module name

    def __init__(self, address: Union[str, RoochAddress], name: str):
        """
        Args:
            address: Address as RoochAddress or hex string
            name: Module name
        """
        # Convert string address to RoochAddress if needed
        if isinstance(address, str):
            self.address = RoochAddress.from_hex_literal(ensure_hex_prefix(address))
        else:
            self.address = address
        self.name = name

    def __str__(self) -> str:
        """String representation of the module ID."""
        return f"{self.address.to_hex_literal()}::{self.name}"

    def serialize(self, serializer: BcsSerializer):
        """Serialize the module ID."""
        serializer.struct(self.address)
        serializer.str(self.name)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'ModuleId':
        """Deserialize a module ID."""
        address = RoochAddress.deserialize(deserializer)
        name = deserializer.str()
        return ModuleId(address=address, name=name)


@dataclass
class FunctionId(Serializable, Deserializable):
    """Represents a Move function identifier."""
    module_id: ModuleId
    function_name: str

    def __str__(self) -> str:
        """String representation of the function ID."""
        return f"{self.module_id}::{self.function_name}"

    def serialize(self, serializer: BcsSerializer):
        """Serialize the function ID."""
        serializer.struct(self.module_id)
        serializer.str(self.function_name)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'FunctionId':
        """Deserialize a function ID."""
        module_id = ModuleId.deserialize(deserializer)
        function_name = deserializer.str()
        return FunctionId(module_id=module_id, function_name=function_name) 