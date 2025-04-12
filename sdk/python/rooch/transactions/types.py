#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from enum import Enum, IntEnum
from typing import Any, Dict, List, Optional, Tuple, Union

from ..utils.hex import ensure_hex_prefix


class AuthenticatorType(IntEnum):
    """Types of transaction authenticators"""
    
    ED25519 = 0
    SECP256K1 = 1
    SECP256R1 = 2
    MULTI_ED25519 = 3
    MULTI_SECP256K1 = 4
    MULTI_SECP256R1 = 5


class TransactionType(IntEnum):
    """Types of Rooch transactions"""
    
    BITCOIN_MOVE_ACTION = 0
    ETHEREUM_MOVE_ACTION = 1
    MOVE_ACTION = 2
    MOVE_MODULE_TRANSACTION = 3
    BITCOIN_BINDING = 4


class MoveAction(IntEnum):
    """Types of Move actions"""
    
    FUNCTION = 0
    SCRIPT = 1


class TransactionArgument:
    """Transaction argument for Move function calls"""
    
    def __init__(self, type_tag: int, value: Any):
        """
        Args:
            type_tag: Type tag (0-10)
            value: Argument value
        """
        self.type_tag = type_tag
        self.value = value
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        return {
            "type_tag": self.type_tag,
            "value": self.value
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TransactionArgument':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            TransactionArgument instance
        """
        return cls(
            type_tag=data.get("type_tag", 0),
            value=data.get("value")
        )


class FunctionArgument:
    """Function argument for Move function calls"""
    
    def __init__(self, function_id: str, ty_args: List[str], args: List[TransactionArgument]):
        """
        Args:
            function_id: Function ID (module::function)
            ty_args: Type arguments
            args: Function arguments
        """
        self.function_id = function_id
        self.ty_args = ty_args
        self.args = args
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        return {
            "function_id": self.function_id,
            "ty_args": self.ty_args,
            "args": [arg.to_dict() for arg in self.args]
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'FunctionArgument':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            FunctionArgument instance
        """
        return cls(
            function_id=data.get("function_id", ""),
            ty_args=data.get("ty_args", []),
            args=[TransactionArgument.from_dict(arg) for arg in data.get("args", [])]
        )


class MoveActionArgument:
    """Move action argument for transactions"""
    
    def __init__(self, action: int, args: Union[FunctionArgument, str]):
        """
        Args:
            action: Move action type (0=FUNCTION, 1=SCRIPT)
            args: Function arguments or script
        """
        self.action = action
        self.args = args
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        if self.action == MoveAction.FUNCTION:
            return {
                "action": self.action,
                "args": self.args.to_dict()
            }
        else:  # SCRIPT
            return {
                "action": self.action,
                "args": self.args
            }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'MoveActionArgument':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            MoveActionArgument instance
        """
        action = data.get("action", 0)
        args = data.get("args")
        
        if action == MoveAction.FUNCTION:
            args = FunctionArgument.from_dict(args)
        
        return cls(action=action, args=args)


class TransactionData:
    """Transaction data for Rooch transactions"""
    
    def __init__(
        self,
        tx_type: int,
        tx_arg: Union[MoveActionArgument, Dict[str, Any], str, bytes],
        sequence_number: int,
        max_gas_amount: int = 10_000_000,
        gas_unit_price: int = 1,
        expiration_timestamp_secs: int = 0,
        chain_id: int = 1
    ):
        """
        Args:
            tx_type: Transaction type
            tx_arg: Transaction arguments
            sequence_number: Transaction sequence number
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_timestamp_secs: Expiration timestamp in seconds
            chain_id: Chain ID
        """
        self.tx_type = tx_type
        self.tx_arg = tx_arg
        self.sequence_number = sequence_number
        self.max_gas_amount = max_gas_amount
        self.gas_unit_price = gas_unit_price
        self.expiration_timestamp_secs = expiration_timestamp_secs
        self.chain_id = chain_id
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        result = {
            "tx_type": self.tx_type,
            "sequence_number": str(self.sequence_number),
            "max_gas_amount": str(self.max_gas_amount),
            "gas_unit_price": str(self.gas_unit_price),
            "expiration_timestamp_secs": str(self.expiration_timestamp_secs),
            "chain_id": self.chain_id,
        }
        
        # Handle different tx_arg types
        if self.tx_type == TransactionType.MOVE_ACTION:
            result["tx_arg"] = self.tx_arg.to_dict() if isinstance(self.tx_arg, MoveActionArgument) else self.tx_arg
        elif self.tx_type == TransactionType.MOVE_MODULE_TRANSACTION:
            if isinstance(self.tx_arg, bytes):
                from ..utils.hex import to_hex
                result["tx_arg"] = to_hex(self.tx_arg)
            else:
                result["tx_arg"] = self.tx_arg
        elif self.tx_type == TransactionType.BITCOIN_BINDING:
            result["tx_arg"] = self.tx_arg
        else:
            result["tx_arg"] = self.tx_arg
        
        return result
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TransactionData':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            TransactionData instance
        """
        tx_type = data.get("tx_type", 0)
        tx_arg = data.get("tx_arg")
        
        # Parse sequence number (can be str or int)
        sequence_number = data.get("sequence_number", 0)
        if isinstance(sequence_number, str):
            sequence_number = int(sequence_number)
        
        # Parse gas amounts (can be str or int)
        max_gas_amount = data.get("max_gas_amount", 10_000_000)
        if isinstance(max_gas_amount, str):
            max_gas_amount = int(max_gas_amount)
            
        gas_unit_price = data.get("gas_unit_price", 1)
        if isinstance(gas_unit_price, str):
            gas_unit_price = int(gas_unit_price)
            
        expiration_timestamp_secs = data.get("expiration_timestamp_secs", 0)
        if isinstance(expiration_timestamp_secs, str):
            expiration_timestamp_secs = int(expiration_timestamp_secs)
        
        # Handle different tx_arg types
        if tx_type == TransactionType.MOVE_ACTION:
            if isinstance(tx_arg, dict):
                tx_arg = MoveActionArgument.from_dict(tx_arg)
        elif tx_type == TransactionType.MOVE_MODULE_TRANSACTION:
            if isinstance(tx_arg, str) and tx_arg.startswith("0x"):
                from ..utils.hex import from_hex
                tx_arg = from_hex(tx_arg)
        
        return cls(
            tx_type=tx_type,
            tx_arg=tx_arg,
            sequence_number=sequence_number,
            max_gas_amount=max_gas_amount,
            gas_unit_price=gas_unit_price,
            expiration_timestamp_secs=expiration_timestamp_secs,
            chain_id=data.get("chain_id", 1)
        )


class AuthenticationKey:
    """Authentication key for transactions"""
    
    def __init__(self, auth_type: int, public_key: Union[str, bytes]):
        """
        Args:
            auth_type: Authentication type
            public_key: Public key (hex string or bytes)
        """
        self.auth_type = auth_type
        
        # Normalize public key
        if isinstance(public_key, str):
            from ..utils.hex import from_hex
            self.public_key = from_hex(ensure_hex_prefix(public_key))
        else:
            self.public_key = public_key
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        from ..utils.hex import to_hex
        return {
            "auth_type": self.auth_type,
            "public_key": to_hex(self.public_key)
        }


class TransactionAuthenticator:
    """Authentication data for transactions"""
    
    def __init__(
        self,
        account_addr: str,
        public_key: Union[str, bytes],
        signature: Union[str, bytes],
        auth_type: int = AuthenticatorType.ED25519
    ):
        """
        Args:
            account_addr: Account address
            public_key: Public key (hex string or bytes)
            signature: Signature (hex string or bytes)
            auth_type: Authentication type
        """
        self.account_addr = account_addr
        self.auth_key = AuthenticationKey(auth_type, public_key)
        
        # Normalize signature
        if isinstance(signature, str):
            from ..utils.hex import from_hex
            self.signature = from_hex(ensure_hex_prefix(signature))
        else:
            self.signature = signature
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        from ..utils.hex import to_hex
        return {
            "account_addr": self.account_addr,
            "auth_key": self.auth_key.to_dict(),
            "signature": to_hex(self.signature)
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TransactionAuthenticator':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            TransactionAuthenticator instance
        """
        auth_key = data.get("auth_key", {})
        auth_type = auth_key.get("auth_type", AuthenticatorType.ED25519)
        public_key = auth_key.get("public_key", "")
        
        return cls(
            account_addr=data.get("account_addr", ""),
            public_key=public_key,
            signature=data.get("signature", ""),
            auth_type=auth_type
        )


class SignedTransaction:
    """Signed transaction ready for submission"""
    
    def __init__(self, tx_data: TransactionData, authenticator: TransactionAuthenticator):
        """
        Args:
            tx_data: Transaction data
            authenticator: Transaction authenticator
        """
        self.tx_data = tx_data
        self.authenticator = authenticator
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        return {
            "tx_data": self.tx_data.to_dict(),
            "authenticator": self.authenticator.to_dict()
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SignedTransaction':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            SignedTransaction instance
        """
        return cls(
            tx_data=TransactionData.from_dict(data.get("tx_data", {})),
            authenticator=TransactionAuthenticator.from_dict(data.get("authenticator", {}))
        )