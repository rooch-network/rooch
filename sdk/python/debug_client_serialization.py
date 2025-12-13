#!/usr/bin/env python3

"""
Debug script to trace the exact serialization path used in client calls
"""

import sys
import os

# Add the project root to the Python path for imports
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

def debug_client_serialization():
    """Debug the exact serialization path used in RoochClient.execute_move_call"""
    
    from rooch.transactions.move.move_types import FunctionArgument, TransactionArgument, MoveActionArgument, MoveAction
    from rooch.transactions.tags.type_tags import TypeTag, StructTag, TypeTagCode
    from rooch.transactions.transaction_types import TransactionData, TransactionType
    from rooch.bcs.serializer import BcsSerializer
    from rooch.address.rooch import RoochAddress
    from rooch.crypto.keypair import KeyPair
    from rooch.crypto.signer import RoochSigner
    
    print("🔍 Debugging client serialization path...")
    
    # Use the exact same data as the failing test
    sender_kp = KeyPair.generate()  # Use any key for testing
    sender_signer = RoochSigner(sender_kp)
    sender_address = sender_signer.get_address()
    
    recipient_address = "0x6912da43ddadbc8d8284264383df036b54f6ea3c2f798d485e7b42bcd6e4d3ab"
    amount = 1
    
    print(f"Sender: {sender_address}")
    print(f"Recipient: {recipient_address}")
    print(f"Amount: {amount}")
    
    # Step 1: Build function argument exactly as in RoochClient.execute_move_call
    function_id = "0x3::transfer::transfer_coin"
    ty_args = [TypeTag.struct(StructTag(address="0x3", module="gas_coin", name="RGas", type_params=[]))]
    args = [recipient_address, amount]
    
    print(f"\n🏗️ Building FunctionArgument...")
    print(f"Function ID: {function_id}")
    print(f"Type args: {ty_args}")
    print(f"Raw args: {args}")
    
    # This is what happens inside RoochClient.execute_move_call
    func_arg = FunctionArgument(function_id, ty_args, args)
    print(f"FunctionArgument created with {len(func_arg.args)} args:")
    for i, arg in enumerate(func_arg.args):
        print(f"  Arg {i}: type={arg.type_tag}, value={arg.value}")
        print(f"         isinstance(TransactionArgument): {isinstance(arg, TransactionArgument)}")
    
    # Step 2: Build MoveActionArgument
    move_action_arg = MoveActionArgument(MoveAction.FUNCTION, func_arg)
    
    # Step 3: Build TransactionData
    tx_data = TransactionData(
        tx_type=TransactionType.MOVE_ACTION,
        tx_arg=move_action_arg,
        sequence_number=46,  # From the test output
        max_gas_amount=10_000_000,
        chain_id=4,
        sender=RoochAddress.from_hex(str(sender_address))
    )
    
    # Step 4: Serialize the full transaction
    print(f"\n🎯 Serializing complete transaction...")
    tx_ser = BcsSerializer()
    tx_data.serialize(tx_ser)
    tx_bytes = tx_ser.output()
    tx_hex = tx_bytes.hex()
    
    print(f"Transaction hex: {tx_hex}")
    
    # Step 5: Analyze the hex data for our specific parameters
    target_addr = "6912da43ddadbc8d8284264383df036b54f6ea3c2f798d485e7b42bcd6e4d3ab"
    target_amount = "0100000000000000000000000000000000000000000000000000000000000000"
    
    addr_pos = tx_hex.find(target_addr)
    amount_pos = tx_hex.find(target_amount)
    
    print(f"\n🔎 Analyzing parameters in hex...")
    print(f"Target address: {target_addr}")
    print(f"Address position in hex: {addr_pos}")
    
    if addr_pos > 2:
        before_addr = tx_hex[addr_pos-2:addr_pos]
        print(f"2 chars before address: {before_addr}")
        if before_addr == "04":
            print("❌ ERROR: Found ADDRESS type tag (04) before address!")
        elif before_addr == "21":
            print("✅ Found length prefix (21 = 33 bytes) before address")
        else:
            print(f"⚠️  Unexpected prefix: {before_addr}")
    
    print(f"\nTarget amount: {target_amount}")
    print(f"Amount position in hex: {amount_pos}")
    
    if amount_pos > 2:
        before_amount = tx_hex[amount_pos-2:amount_pos]
        print(f"2 chars before amount: {before_amount}")
        if before_amount == "0a":
            print("❌ ERROR: Found U256 type tag (0a) before amount!")
        elif before_amount == "21":
            print("✅ Found length prefix (21 = 33 bytes) before amount")
        else:
            print(f"⚠️  Unexpected prefix: {before_amount}")
    
    # Step 6: Test individual component serialization
    print(f"\n🧪 Testing component serialization...")
    
    # Test FunctionArgument alone
    func_ser = BcsSerializer()
    func_arg.serialize(func_ser)
    func_bytes = func_ser.output()
    func_hex = func_bytes.hex()
    
    print(f"FunctionArgument hex: {func_hex}")
    
    func_addr_pos = func_hex.find(target_addr)
    func_amount_pos = func_hex.find(target_amount)
    
    if func_addr_pos > 2:
        func_before_addr = func_hex[func_addr_pos-2:func_addr_pos]
        print(f"FunctionArgument - 2 chars before address: {func_before_addr}")
    
    if func_amount_pos > 2:
        func_before_amount = func_hex[func_amount_pos-2:func_amount_pos]
        print(f"FunctionArgument - 2 chars before amount: {func_before_amount}")
    
    # Test one individual argument
    print(f"\n🔬 Testing individual TransactionArgument serialization...")
    first_arg = func_arg.args[0]
    
    # Test with type tag (original serialize method)
    arg_ser_with_tag = BcsSerializer()
    first_arg.serialize(arg_ser_with_tag)
    arg_with_tag = arg_ser_with_tag.output().hex()
    print(f"First arg with type tag: {arg_with_tag}")
    
    # Test without type tag (our new serialize_value_only method)
    arg_ser_no_tag = BcsSerializer()
    first_arg.serialize_value_only(arg_ser_no_tag)
    arg_no_tag = arg_ser_no_tag.output().hex()
    print(f"First arg without type tag: {arg_no_tag}")
    
    # Verify which one is actually used in the sequence
    # By checking if the func_hex contains the with-tag or no-tag version
    if arg_with_tag in func_hex:
        print("❌ FunctionArgument is using serialize() with type tags!")
    elif arg_no_tag in func_hex:
        print("✅ FunctionArgument is using serialize_value_only() without type tags!")
    else:
        print("❓ Neither version found - something else is happening")

if __name__ == "__main__":
    try:
        debug_client_serialization()
    except Exception as e:
        print(f"❌ Debug failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
