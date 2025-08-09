#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import pytest
from rooch.transactions.builder import TransactionBuilder
from rooch.crypto.signer import RoochSigner
from rooch.transactions.types import (
    MoveActionArgument, 
    MoveAction, 
    FunctionArgument,
    ModuleId,
    FunctionId,
    TransactionData,
    TransactionType
)
from rooch.address.rooch import RoochAddress
from rooch.address.bitcoin import BitcoinAddress


def test_basic_signer_creation():
    """Test that we can create a signer and get addresses"""
    signer = RoochSigner.generate()
    rooch_address = signer.get_address()
    
    # Should be a valid hex address
    assert rooch_address.startswith('0x')
    assert len(rooch_address) == 66  # 0x + 64 hex chars
    
    # Should be able to get Bitcoin address
    keypair = signer.get_keypair()
    public_key_bytes = keypair.get_public_key()
    bitcoin_address = BitcoinAddress.from_public_key(public_key_bytes, mainnet=True)
    bitcoin_address_str = str(bitcoin_address)
    
    # Bitcoin address should be valid
    assert len(bitcoin_address_str) > 20
    # Print for debug
    print(f"Rooch address: {rooch_address}")
    print(f"Bitcoin address: {bitcoin_address_str}")

def test_transaction_builder_with_bitcoin_auth():
    """Test that TransactionBuilder can create and sign transactions with Bitcoin authenticator"""
    signer = RoochSigner.generate()
    
    # Create transaction builder
    builder = TransactionBuilder(
        sender_address=signer.get_address(),
        sequence_number=0,
        chain_id=4,
        max_gas_amount=10_000_000,
    )
    
    # Build a simple function payload
    mod_id = ModuleId(address="0x3", name="transfer")
    func_id = FunctionId(module_id=mod_id, function_name="transfer_coin")
    func_arg = FunctionArgument(function_id=func_id, ty_args=[], args=[])
    payload = MoveActionArgument(MoveAction.FUNCTION, func_arg)
    
    # Build transaction data
    tx_data = builder.build_move_action_tx(payload)
    
    # Sign the transaction (this should use Bitcoin authenticator)
    signed_tx = builder.sign(tx_data, signer)
    
    # Verify the signed transaction structure
    assert signed_tx.tx_data == tx_data
    assert signed_tx.authenticator is not None
    assert signed_tx.authenticator.auth_validator_id == 1  # Bitcoin auth validator ID
    assert len(signed_tx.authenticator.payload) > 0
    
    print(f"Successfully created signed transaction with Bitcoin authenticator")
    print(f"Auth validator ID: {signed_tx.authenticator.auth_validator_id}")
    print(f"Payload length: {len(signed_tx.authenticator.payload)}")

def test_bitcoin_address_generation():
    """Test Bitcoin address generation from secp256k1 public key"""
    signer = RoochSigner.generate()
    keypair = signer.get_keypair()
    public_key_bytes = keypair.get_public_key()
    
    # Should be uncompressed format (65 bytes: 0x04 + X + Y)
    assert len(public_key_bytes) == 65
    assert public_key_bytes[0] == 0x04
    
    # Generate Bitcoin addresses for both mainnet and testnet
    mainnet_addr = BitcoinAddress.from_public_key(public_key_bytes, mainnet=True)
    testnet_addr = BitcoinAddress.from_public_key(public_key_bytes, mainnet=False)
    
    print(f"Public key length: {len(public_key_bytes)}")
    print(f"Mainnet Bitcoin address: {mainnet_addr}")
    print(f"Testnet Bitcoin address: {testnet_addr}")
