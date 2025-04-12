#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from enum import Enum, IntEnum
from typing import Any, Dict, List, Optional, Tuple, Union
from dataclasses import dataclass

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
    
    SCRIPT = 0
    FUNCTION = 1
    MODULE_BUNDLE = 2


class FunctionArgument:
    """Function argument for Move function calls"""
    
    def __init__(self, function_id: 'FunctionId', ty_args: List['TypeTag'], args: List[Any]):
        """
        Args:
            function_id: FunctionId object
            ty_args: List of TypeTag objects
            args: Function arguments (raw values)
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
            "function_id": f"{self.function_id.module_id.address}::{self.function_id.module_id.name}::{self.function_id.function_name}",
            "ty_args": [str(tag) for tag in self.ty_args],
            "args": self.args
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'FunctionArgument':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            FunctionArgument instance
        """
        raise NotImplementedError("from_dict for updated FunctionArgument not implemented")


class MoveActionArgument:
    """Move action argument for transactions"""
    
    def __init__(self, action: MoveAction, args: Union[FunctionArgument, bytes, List[bytes]]):
        """
        Args:
            action: Move action type (SCRIPT, FUNCTION, MODULE_BUNDLE)
            args: Function arguments, script bytecode, or list of module bytecodes
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
        elif self.action == MoveAction.SCRIPT:
            return {
                "action": self.action,
                "args": self.args
            }
        elif self.action == MoveAction.MODULE_BUNDLE:
            return {
                "action": self.action,
                "args": [to_hex(arg) for arg in self.args]
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
        elif action == MoveAction.SCRIPT:
            args = args
        elif action == MoveAction.MODULE_BUNDLE:
            args = [from_hex(arg) for arg in args]
        
        return cls(action=action, args=args)


class TransactionData:
    """Transaction data for Rooch transactions"""
    
    def __init__(
        self,
        sender: str,
        sequence_number: int,
        chain_id: int,
        max_gas_amount: int,
        action: MoveActionArgument
    ):
        """
        Args:
            sender: Sender account address (RoochAddress string)
            sequence_number: Transaction sequence number (u64)
            chain_id: Chain ID (u64)
            max_gas_amount: Maximum gas amount (u64)
            action: The MoveActionArgument to execute
        """
        self.sender = sender
        self.sequence_number = sequence_number
        self.chain_id = chain_id
        self.max_gas_amount = max_gas_amount
        self.action = action
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        result = {
            "sender": self.sender,
            "sequence_number": str(self.sequence_number),
            "chain_id": self.chain_id,
            "max_gas_amount": str(self.max_gas_amount),
            "action": self.action.to_dict()
        }
        
        return result
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TransactionData':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            TransactionData instance
        """
        sender = data.get("sender", "")
        sequence_number = data.get("sequence_number", 0)
        chain_id = data.get("chain_id", 1)
        max_gas_amount = data.get("max_gas_amount", 10_000_000)
        action = MoveActionArgument.from_dict(data.get("action", {}))
        
        return cls(
            sender=sender,
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount,
            action=action
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
        # Restore the public_key assignment
        if isinstance(public_key, str):
            from ..utils.hex import from_hex, ensure_hex_prefix
            self.public_key = from_hex(ensure_hex_prefix(public_key))
        else:
            self.public_key = public_key
        # Normalize and store signature
        if isinstance(signature, str):
            from ..utils.hex import from_hex
            self.signature = from_hex(ensure_hex_prefix(signature))
        else:
            self.signature = signature
        # Store the auth_type
        self.auth_type = auth_type
    
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


# --- TypeTag Definitions ---
from dataclasses import dataclass

# Assuming Address type is available or needs import
# from ..address.rooch import RoochAddress

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
class StructTag:
    # Assuming address string format is acceptable here, BCS will handle conversion
    address: str
    module: str
    name: str
    type_params: List['TypeTag']

@dataclass
class TypeTag:
    type_code: TypeTagCode
    # value holds inner type for Vector, or StructTag for Struct
    value: Optional[Union['TypeTag', StructTag]] = None

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

# --- End TypeTag Definitions ---

# --- ModuleId and FunctionId Definitions ---
@dataclass
class ModuleId:
    address: str # Hex string address
    name: str    # Module name

@dataclass
class FunctionId:
    module_id: ModuleId
    function_name: str
# --- End ModuleId and FunctionId Definitions ---


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