#!/usr/bin/env python3

"""
Debug script to trace the exact serialization path
"""

import sys
import os

# Add the project root to the Python path for imports
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

def debug_serialization_path():
    """Debug the exact serialization path for our transaction"""
    
    from rooch.transactions.move.move_types import FunctionArgument, TransactionArgument, MoveActionArgument, MoveAction
    from rooch.transactions.tags.type_tags import TypeTag, StructTag, TypeTagCode
    from rooch.transactions.transaction_types import TransactionData, TransactionType
    from rooch.bcs.serializer import BcsSerializer
    from rooch.address.rooch import RoochAddress
    
    print("🔍 Debugging serialization path...")
    
    # Create test data exactly as in the failed transaction
    recipient_addr = "0xa5caa06e96bb751331f423d1306b27a808834d93975954fc824f87021cb61085"
    amount = 1
    
    # Build FunctionArgument
    function_id = "0x3::transfer::transfer_coin"
    ty_args = [TypeTag.struct(StructTag(address="0x3", module="gas_coin", name="RGas", type_params=[]))]
    args = [recipient_addr, amount]
    
    func_arg = FunctionArgument(function_id, ty_args, args)
    
    print(f"Created FunctionArgument with {len(func_arg.args)} args:")
    for i, arg in enumerate(func_arg.args):
        print(f"  Arg {i}: type={arg.type_tag}, value={arg.value}")
    
    # Test FunctionArgument serialization directly
    print(f"\n🧪 Testing FunctionArgument.serialize()...")
    func_ser = BcsSerializer()
    func_arg.serialize(func_ser)
    func_bytes = func_ser.output()
    print(f"FunctionArgument bytes: {func_bytes.hex()}")
    
    # Look for the problematic sequence (args part)
    # We expect to see the args sequence without type tag prefixes
    args_part = func_bytes.hex()[-130:]  # Last ~65 bytes should be the args
    print(f"Args part (last 65 bytes): {args_part}")
    
    # Check if this contains type tag prefixes
    if args_part.startswith("02"):  # 2 items in sequence
        remaining = args_part[2:]  # Skip sequence length
        print(f"After sequence length: {remaining}")
        
        # First arg should be 32 bytes address (64 hex chars)
        if len(remaining) >= 64:
            first_arg = remaining[:64]
            print(f"First arg (address): {first_arg}")
            
            if len(remaining) >= 128:
                second_arg = remaining[64:128]
                print(f"Second arg (amount): {second_arg}")
    
    # Test individual arg serialization
    print(f"\n🔬 Testing individual argument serialization...")
    
    addr_arg = func_arg.args[0]
    amount_arg = func_arg.args[1]
    
    # Test with type tags
    addr_ser_with_tag = BcsSerializer()
    addr_arg.serialize(addr_ser_with_tag)
    addr_with_tag = addr_ser_with_tag.output()
    print(f"Address with type tag: {addr_with_tag.hex()}")
    
    # Test without type tags
    addr_ser_no_tag = BcsSerializer()
    addr_arg.serialize_value_only(addr_ser_no_tag)
    addr_no_tag = addr_ser_no_tag.output()
    print(f"Address without type tag: {addr_no_tag.hex()}")
    
    # Test amount
    amount_ser_with_tag = BcsSerializer()
    amount_arg.serialize(amount_ser_with_tag)
    amount_with_tag = amount_ser_with_tag.output()
    print(f"Amount with type tag: {amount_with_tag.hex()}")
    
    amount_ser_no_tag = BcsSerializer()
    amount_arg.serialize_value_only(amount_ser_no_tag)
    amount_no_tag = amount_ser_no_tag.output()
    print(f"Amount without type tag: {amount_no_tag.hex()}")
    
    # Now test MoveActionArgument serialization
    print(f"\n📦 Testing MoveActionArgument.serialize()...")
    move_action_arg = MoveActionArgument(MoveAction.FUNCTION, func_arg)
    
    move_ser = BcsSerializer()
    move_action_arg.serialize(move_ser)
    move_bytes = move_ser.output()
    print(f"MoveActionArgument bytes: {move_bytes.hex()}")
    
    # Finally test TransactionData serialization
    print(f"\n📋 Testing TransactionData.serialize()...")
    
    sender = RoochAddress.from_hex("0x1bb5f31f040703fd8924871dfd8ec4a02d7f2053c2a0faff8ce4164dd005752e")
    tx_data = TransactionData(
        tx_type=TransactionType.MOVE_ACTION,
        tx_arg=move_action_arg,
        sequence_number=42,
        max_gas_amount=10000000,
        chain_id=4,
        sender=sender
    )
    
    tx_ser = BcsSerializer()
    tx_data.serialize(tx_ser)
    tx_bytes = tx_ser.output()
    print(f"TransactionData bytes: {tx_bytes.hex()}")
    
    # Extract the args part from the full transaction
    print(f"\n🎯 Looking for args in full transaction...")
    # The transaction structure is: type(1) + move_action(1) + function_arg + seq(8) + gas(8) + chain(1) + sender(33)
    # So args should be in the function_arg part
    
    expected_addr_hex = "a5caa06e96bb751331f423d1306b27a808834d93975954fc824f87021cb61085"
    expected_amount_hex = "0100000000000000000000000000000000000000000000000000000000000000"
    
    tx_hex = tx_bytes.hex()
    if expected_addr_hex in tx_hex and expected_amount_hex in tx_hex:
        print("✅ Found expected address and amount in transaction")
        
        # Find their positions
        addr_pos = tx_hex.find(expected_addr_hex)
        amount_pos = tx_hex.find(expected_amount_hex)
        
        print(f"Address position: {addr_pos}, Amount position: {amount_pos}")
        
        # Check what's before the address (should not be type tag)
        if addr_pos > 2:
            before_addr = tx_hex[addr_pos-2:addr_pos]
            print(f"2 chars before address: {before_addr}")
            if before_addr == "04":
                print("❌ Found type tag (04) before address!")
            else:
                print("✅ No type tag before address")
        
        # Check what's before the amount
        if amount_pos > 2:
            before_amount = tx_hex[amount_pos-2:amount_pos]
            print(f"2 chars before amount: {before_amount}")
            if before_amount == "0a":
                print("❌ Found type tag (0a) before amount!")
            else:
                print("✅ No type tag before amount")
    else:
        print("❌ Could not find expected address and amount in transaction")

if __name__ == "__main__":
    try:
        debug_serialization_path()
    except Exception as e:
        print(f"❌ Debug failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
