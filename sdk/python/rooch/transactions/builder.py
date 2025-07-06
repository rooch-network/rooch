#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import time
from typing import Any, Dict, List, Optional, Union
import hashlib

from ..address.rooch import RoochAddress
from ..crypto.signer import Signer
from ..crypto.signer import RoochSigner
from ..utils.hex import from_hex
from .serializer import TxSerializer
from .types import (
    FunctionArgument,
    MoveAction,
    MoveActionArgument,
    SignedTransaction,
    TransactionAuthenticator,
    TransactionData,
    TransactionType,
    ModuleId,
    FunctionId,
    TypeTag,
    StructTag
)


class TransactionBuilder:
    """Builder for Rooch transactions"""
    
    def __init__(
        self,
        sender_address: str,
        sequence_number: int,
        chain_id: int,
        max_gas_amount: int = 10_000_000,
    ):
        """Initialize a transaction builder based on new TransactionData fields
        
        Args:
            sender_address: Sender account address
            sequence_number: Transaction sequence number
            chain_id: Chain ID
            max_gas_amount: Maximum gas amount
        """
        self.sender_address = sender_address
        self.sequence_number = sequence_number
        self.chain_id = chain_id
        self.max_gas_amount = max_gas_amount
    
    def build_function_payload(
        self,
        function_id: str,
        ty_args: Optional[List[Union[str, TypeTag]]] = None,
        args: Optional[List[Any]] = None
    ) -> MoveActionArgument:
        """Build a function payload for MoveAction
        
        Args:
            function_id: Function ID (module::function)
            ty_args: Optional type arguments (can be strings or TypeTag objects)
            args: Optional list of raw argument values
            
        Returns:
            MoveActionArgument for function call
        """
        ty_args = ty_args or []
        args = args or []
        
        # Convert string type args to TypeTag objects
        converted_ty_args = []
        for ty_arg in ty_args:
            if isinstance(ty_arg, str):
                # Parse string like "0x3::gas_coin::RGas"
                parts = ty_arg.split("::")
                if len(parts) != 3:
                    raise ValueError(f"Invalid type argument format: {ty_arg}")
                addr_str, module, name = parts
                struct_tag = StructTag(address=addr_str, module=module, name=name, type_params=[])
                converted_ty_args.append(TypeTag.struct(struct_tag))
            elif isinstance(ty_arg, TypeTag):
                converted_ty_args.append(ty_arg)
            else:
                raise TypeError(f"Invalid type argument: {ty_arg}")
        
        try:
            parts = function_id.split("::")
            if len(parts) != 3:
                 raise ValueError("Invalid function_id format. Expected 'address::module::function'")
            addr_str, module_name, func_name = parts
            if addr_str.startswith('0x') and len(addr_str) < 66:
                try:
                    int(addr_str[2:], 16)
                except ValueError:
                    raise ValueError(f"Invalid short hex address: {addr_str}")
            
            mod_id = ModuleId(address=addr_str, name=module_name)
            func_id_obj = FunctionId(module_id=mod_id, function_name=func_name)
        except Exception as e:
            raise ValueError(f"Failed to parse function_id '{function_id}': {e}") from e
            
        func_arg = FunctionArgument(function_id=func_id_obj, ty_args=converted_ty_args, args=args)
        
        return MoveActionArgument(MoveAction.FUNCTION, func_arg)
    
    def build_move_action_tx(self, action_arg: MoveActionArgument) -> TransactionData:
        """Build a Move action transaction
        
        Args:
            action_arg: Move action argument
            
        Returns:
            TransactionData for the action
        """
        return TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=action_arg,
            sequence_number=self.sequence_number,
            max_gas_amount=self.max_gas_amount,
            chain_id=self.chain_id,
            sender=RoochAddress.from_hex(self.sender_address)
        )
    
    def build_module_publish_tx(self, module_bytes: Union[bytes, str]) -> TransactionData:
        """Build a module publish transaction
        
        Args:
            module_bytes: Move module bytecode (bytes or hex string)
            
        Returns:
            TransactionData for the module publish
        """
        if isinstance(module_bytes, str):
            module_bytes = from_hex(module_bytes)
        elif not isinstance(module_bytes, bytes):
            raise TypeError("module_bytes must be bytes or hex string")

        return TransactionData(
            tx_type=TransactionType.MOVE_MODULE_TRANSACTION,
            tx_arg=module_bytes,
            sequence_number=self.sequence_number,
            max_gas_amount=self.max_gas_amount,
            chain_id=self.chain_id,
            sender=RoochAddress.from_hex(self.sender_address)
        )
    
    def sign(self, tx_data: TransactionData, signer: Signer) -> SignedTransaction:
        """Sign transaction data and create a signed transaction"""
        # 1. Serialize TransactionData
        serialized_tx_data = TxSerializer.encode_transaction_data(tx_data)

        # 2. Calculate SHA3-256 hash of serialized data
        tx_hash_bytes = hashlib.sha3_256(serialized_tx_data).digest()
        
        # 3. Access the KeyPair from the Signer (assuming RoochSigner)
        if not isinstance(signer, RoochSigner):
             # We might need a more generic way if other Signer types exist
             raise TypeError("Expected RoochSigner to access KeyPair")
        keypair = signer.get_keypair()
        
        # Get public key and generate Bitcoin address
        public_key_bytes = keypair.get_public_key()  # 65 bytes uncompressed
        
        # Create compressed public key (33 bytes) for Bitcoin auth
        x_coord = public_key_bytes[1:33]  # Extract x coordinate
        y_coord = public_key_bytes[33:65]  # Extract y coordinate
        
        # Determine if y is even or odd to set the prefix
        y_int = int.from_bytes(y_coord, byteorder='big')
        if y_int % 2 == 0:
            compressed_public_key = b'\x02' + x_coord
        else:
            compressed_public_key = b'\x03' + x_coord
        
        from ..address.bitcoin import BitcoinAddress
        bitcoin_address = BitcoinAddress.from_public_key(public_key_bytes, mainnet=True)
        bitcoin_address_str = str(bitcoin_address)
        
        # Debug info
        print(f"Debug - Public key: {public_key_bytes.hex()}")
        print(f"Debug - Compressed public key: {compressed_public_key.hex()}")
        print(f"Debug - Bitcoin address: {bitcoin_address_str}")
        print(f"Debug - TX hash: {tx_hash_bytes.hex()}")
        
        # Create SignData following Rust implementation
        # Rust constants: MESSAGE_INFO_PREFIX = b"Bitcoin Signed Message:\n" (NO \x18!), MESSAGE_INFO = b"Rooch Transaction:\n"
        # Note: Rust removes \x18 because consensus codec already includes length info
        message_prefix = b"Bitcoin Signed Message:\n"  # NO \x18 prefix
        message_info_without_tx_hash = b"Rooch Transaction:\n"
        
        # Create SignData with tx_hash appended (like Rust SignData::new())
        tx_hash_hex = tx_hash_bytes.hex().encode('utf-8')  # Convert to bytes
        message_info = message_info_without_tx_hash + tx_hash_hex
        
        # Encode varint for Bitcoin consensus encoding (matching Move consensus_codec)
        def encode_varint(n):
            if n <= 0xFC:  # Move/Bitcoin uses 0xFC (252) as threshold
                return bytes([n])
            elif n <= 0xFFFF:
                return b'\xfd' + n.to_bytes(2, 'little')
            elif n <= 0xFFFFFFFF:
                return b'\xfe' + n.to_bytes(4, 'little')
            else:
                return b'\xff' + n.to_bytes(8, 'little')
        
        # Bitcoin consensus encoding format (like Rust SignData::consensus_encode)
        # Each vector is encoded as: varint(length) + data
        prefix_encoded = encode_varint(len(message_prefix)) + message_prefix
        info_encoded = encode_varint(len(message_info)) + message_info
        sign_data_encoded = prefix_encoded + info_encoded
        
        # Move does: hash::sha2_256(message) - first SHA256
        first_hash = hashlib.sha256(sign_data_encoded).digest()
        
        # Rust ecdsa_k1::verify does verify_with_hash::<Sha256> - second SHA256
        # So we need to sign the double SHA256 hash
        sign_data_hash = hashlib.sha256(first_hash).digest()
        
        # Debug info for signing
        print(f"Debug - Sign data encoded length: {len(sign_data_encoded)}")
        print(f"Debug - Sign data encoded: {sign_data_encoded.hex()}")
        print(f"Debug - First SHA256: {first_hash.hex()}")
        print(f"Debug - Second SHA256 (final hash): {sign_data_hash.hex()}")

        # IMPORTANT: Sign the double-hashed data to match Rust verify_with_hash::<Sha256>
        signature_bytes = keypair.sign_digest(sign_data_hash)
        
        # Debug: verify signature locally using compressed public key (like Move does)
        try:
            from ecdsa import VerifyingKey, SECP256k1
            from ecdsa.util import sigdecode_string
            # Use UNCOMPRESSED public key for local verification first 
            vk = VerifyingKey.from_string(public_key_bytes[1:], curve=SECP256k1)  # Remove 0x04 prefix
            # Use verify_digest with the SAME double-hashed data that we signed
            is_valid = vk.verify_digest(signature_bytes, sign_data_hash, sigdecode=sigdecode_string)
            print(f"Debug - Local signature verification (uncompressed key): {is_valid}")
        except Exception as e:
            print(f"Debug - Local verification failed: {e}")

        print(f"Debug - Signature: {signature_bytes.hex()}")

        # Create AuthPayload structure as a proper struct (following Rust AuthPayload)
        # Rust struct fields: signature, message_prefix, message_info, public_key, from_address
        from ..bcs.serializer import BcsSerializer, Serializable
        
        class BitcoinAuthPayload(Serializable):
            """Bitcoin AuthPayload structure matching Rust implementation"""
            def __init__(self, signature: bytes, message_prefix: bytes, message_info: bytes, 
                        public_key: bytes, from_address: bytes):
                self.signature = signature
                self.message_prefix = message_prefix
                self.message_info = message_info
                self.public_key = public_key
                self.from_address = from_address
            
            def serialize(self, serializer: BcsSerializer):
                """Serialize using BCS struct protocol"""
                serializer.bytes(self.signature)
                serializer.bytes(self.message_prefix)
                serializer.bytes(self.message_info)
                serializer.bytes(self.public_key)
                serializer.bytes(self.from_address)
        
        # Create the AuthPayload struct
        auth_payload = BitcoinAuthPayload(
            signature=signature_bytes,
            message_prefix=message_prefix,  # raw prefix without varint
            message_info=message_info_without_tx_hash,  # without tx hash
            public_key=compressed_public_key,  # Use compressed public key (33 bytes)
            from_address=bitcoin_address_str.encode('utf-8')
        )
        
        # Serialize the entire struct using BCS struct protocol
        serializer = BcsSerializer()
        serializer.struct(auth_payload)
        auth_payload_bytes = serializer.output()
        auth = TransactionAuthenticator.bitcoin(auth_payload_bytes)
        # 6. Return SignedTransaction
        return SignedTransaction(tx_data, auth)

    def build_signed_tx(
        self,
        payload: MoveActionArgument,
        signer: Signer
    ) -> SignedTransaction:
        # ... existing logic ...
        tx_data = self.build_move_action_tx(payload)
        return self.sign(tx_data, signer)

    @classmethod
    def with_default_account(
        cls,
        signer: Signer,
        sequence_number: int,
        chain_id: int,
        max_gas_amount: int = 10_000_000,
    ) -> 'TransactionBuilder':
        """Create a transaction builder with default account settings
        
        Args:
            signer: Transaction signer
            sequence_number: Transaction sequence number
            chain_id: Chain ID
            max_gas_amount: Maximum gas amount
            
        Returns:
            TransactionBuilder instance
        """
        return cls(
            sender_address=signer.get_address(),
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount,
        )