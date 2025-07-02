#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import time
from typing import Any, Dict, List, Optional, Union

from ..address.rooch import RoochAddress
from ..crypto.keypair import KeyPair
from ..crypto.signer import Signer, RoochSigner
from ..transactions.builder import TransactionBuilder
from ..transactions.types import FunctionArgument, MoveAction, MoveActionArgument, TypeTag, ModuleId, FunctionId, StructTag
from ..bcs.serializer import Args
from ..utils.hex import from_hex

# Constants
DEFAULT_MAX_INACTIVE_INTERVAL = 1200  # seconds
REQUIRED_SCOPE = "0x3::session_key::remove_session_key_entry"

class CreateSessionArgs:
    """Arguments for creating a session."""
    def __init__(
        self,
        app_name: str,
        app_url: str,
        scopes: List[str],
        max_inactive_interval: Optional[int] = None,
        keypair: Optional[KeyPair] = None,
    ):
        self.app_name = app_name
        self.app_url = app_url
        self.scopes = scopes
        self.max_inactive_interval = max_inactive_interval or DEFAULT_MAX_INACTIVE_INTERVAL
        self.keypair = keypair or KeyPair.generate()

class Session(RoochSigner):
    """Represents an active session key for signing transactions."""

    def __init__(
        self,
        app_name: str,
        app_url: str,
        scopes: List[str],
        rooch_address: RoochAddress,
        keypair: KeyPair,
        max_inactive_interval: int,
        local_create_session_time: Optional[int] = None,
        last_active_time: Optional[int] = None,
    ):
        super().__init__(keypair)
        self.app_name = app_name
        self.app_url = app_url
        self.scopes = scopes
        self.rooch_address = rooch_address
        self.max_inactive_interval = max_inactive_interval
        self.local_create_session_time = local_create_session_time or int(time.time())
        self.last_active_time = last_active_time or self.local_create_session_time

    @classmethod
    async def create(
        cls,
        client: Any, # Use Any to avoid circular import with RoochClient
        signer: Signer,
        session_args: CreateSessionArgs,
    ) -> "Session":
        """Creates a new session key on the Rooch network."""
        parsed_scopes = []
        for scope in session_args.scopes:
            if isinstance(scope, str):
                if scope.count("::") != 2:
                    raise ValueError("Invalid scope format. Expected 'address::module::function' or 'address::module::*' or 'address::*::*'")
                parsed_scopes.append(scope)
            else:
                raise TypeError(f"Invalid scope type: {type(scope)}")

        all_ox3 = "0x3::*::*"
        if all_ox3 not in parsed_scopes and REQUIRED_SCOPE not in parsed_scopes:
            parsed_scopes.append(REQUIRED_SCOPE)

        session_keypair = session_args.keypair
        session_rooch_address = session_keypair.get_rooch_address()

        # Build the transaction to create the session key
        tx_builder = await client.get_transaction_builder(
            sender_address=signer.get_address(),
            signer=signer, # Use the original signer for the transaction
            max_gas_amount=10_000_000,
        )

        # Split scopes into addresses, modules, functions
        addrs = []
        mods = []
        fns = []
        for scope in parsed_scopes:
            parts = scope.split("::")
            addrs.append(parts[0])
            mods.append(parts[1])
            fns.append(parts[2])

        payload = tx_builder.build_function_payload(
            function_id="0x3::session_key::create_session_key_with_multi_scope_entry",
            ty_args=[],
            args=[
                Args.string(session_args.app_name),
                Args.string(session_args.app_url),
                Args.vector_u8(from_hex(session_keypair.get_rooch_address().to_hex())),
                Args.vector_address(addrs),
                Args.vector_string(mods),
                Args.vector_string(fns),
                Args.u64(session_args.max_inactive_interval),
            ],
        )

        tx_data = tx_builder.build_move_action_tx(payload)
        signed_tx = tx_builder.sign(tx_data, signer)

        result = await client.submit_and_wait(signed_tx)

        if result.get("execution_info", {}).get("status", {}).get("type") == "executed":
            return cls(
                app_name=session_args.app_name,
                app_url=session_args.app_url,
                scopes=parsed_scopes,
                rooch_address=session_rooch_address,
                keypair=session_keypair,
                max_inactive_interval=session_args.max_inactive_interval,
            )
        else:
            raise Exception(f"Create session failed: {result.get('execution_info', {}).get('status')}")

    def sign_transaction(self, transaction_data: Any) -> Any:
        """Sign a transaction using the session keypair."""
        # Update last active time
        self.last_active_time = int(time.time())
        return super().sign_transaction(transaction_data)

    def is_session_expired(self) -> bool:
        """Checks if the session has expired based on max_inactive_interval."""
        expiration_time = self.last_active_time + self.max_inactive_interval
        return int(time.time()) > expiration_time

    def to_dict(self) -> Dict[str, Any]:
        """Converts the session object to a dictionary."""
        return {
            "app_name": self.app_name,
            "app_url": self.app_url,
            "scopes": self.scopes,
            "rooch_address": str(self.rooch_address),
            "public_key": self.get_public_key_hex(),
            "private_key": self.get_private_key_hex(), # Include private key for serialization/deserialization
            "max_inactive_interval": self.max_inactive_interval,
            "local_create_session_time": self.local_create_session_time,
            "last_active_time": self.last_active_time,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Session":
        """Creates a Session object from a dictionary."""
        keypair = KeyPair.from_private_key(data["private_key"])
        rooch_address = RoochAddress.from_hex(data["rooch_address"])
        return cls(
            app_name=data["app_name"],
            app_url=data["app_url"],
            scopes=data["scopes"],
            rooch_address=rooch_address,
            keypair=keypair,
            max_inactive_interval=data["max_inactive_interval"],
            local_create_session_time=data.get("local_create_session_time"),
            last_active_time=data.get("last_active_time"),
        )
