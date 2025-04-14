#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from dataclasses import dataclass
from typing import Union

from ...address.rooch import RoochAddress
from ...utils.hex import ensure_hex_prefix


@dataclass
class ModuleId:
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


@dataclass
class FunctionId:
    """Represents a Move function identifier."""
    module_id: ModuleId
    function_name: str

    def __str__(self) -> str:
        """String representation of the function ID."""
        return f"{self.module_id}::{self.function_name}" 