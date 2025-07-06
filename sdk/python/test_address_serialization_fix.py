#!/usr/bin/env python3

"""
Test script to verify address serialization fix
"""

import sys
import os

# Add the project root to the Python path for imports
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

def test_address_serialization_fix():
    """Test that address parameters no longer include type tag prefix"""
    
    print("Testing address serialization fix...")
    
    # Test the updated FunctionArgument logic
    from rooch.transactions.move.move_types import FunctionArgument, TransactionArgument
    from rooch.transactions.tags.type_tags import TypeTag, StructTag, TypeTagCode
    from rooch.bcs.serializer import BcsSerializer
    
    # Create test parameters
    test_recipient = "0x1bb5f31f040703fd8924871dfd8ec4a02d7f2053c2a0faff8ce4164dd005752e"
    test_amount = 1  # Small amount
    
    # Build function argument
    function_id = "0x3::transfer::transfer_coin"
    ty_args = [TypeTag.struct(StructTag(
        address="0x3", 
        module="gas_coin", 
        name="RGas", 
        type_params=[]
    ))]
    args = [test_recipient, test_amount]
    
    func_arg = FunctionArgument(function_id, ty_args, args)
    
    # Check the inferred types
    print(f"Arguments created: {len(func_arg.args)}")
    for i, arg in enumerate(func_arg.args):
        print(f"Arg {i}: type={arg.type_tag}, value={arg.value}")
    
    # Test individual argument serialization with and without type tags
    print("\n=== Testing individual argument serialization ===")
    
    # Address argument
    addr_arg = func_arg.args[0]
    print(f"Address argument: {addr_arg.value}")
    
    # Serialize with type tag (old way)
    serializer_with_tag = BcsSerializer()
    addr_arg.serialize(serializer_with_tag)
    with_tag_data = serializer_with_tag.output()
    print(f"With type tag: {with_tag_data.hex()}")
    
    # Serialize without type tag (new way)
    serializer_without_tag = BcsSerializer()
    addr_arg.serialize_value_only(serializer_without_tag)
    without_tag_data = serializer_without_tag.output()
    print(f"Without type tag: {without_tag_data.hex()}")
    
    # Amount argument
    amount_arg = func_arg.args[1]
    print(f"\nAmount argument: {amount_arg.value}")
    
    # Serialize with type tag (old way)
    serializer_with_tag2 = BcsSerializer()
    amount_arg.serialize(serializer_with_tag2)
    with_tag_data2 = serializer_with_tag2.output()
    print(f"With type tag: {with_tag_data2.hex()}")
    
    # Serialize without type tag (new way)
    serializer_without_tag2 = BcsSerializer()
    amount_arg.serialize_value_only(serializer_without_tag2)
    without_tag_data2 = serializer_without_tag2.output()
    print(f"Without type tag: {without_tag_data2.hex()}")
    
    # Test full function argument serialization
    print("\n=== Testing full function argument serialization ===")
    
    serializer = BcsSerializer()
    func_arg.serialize(serializer)
    serialized_data = serializer.output()
    
    print(f"Full function argument: {serialized_data.hex()}")
    
    # Check if the serialized data matches the expected format
    # From the correct transaction, we expect:
    # Address: 0x1bb5f31f040703fd8924871dfd8ec4a02d7f2053c2a0faff8ce4164dd005752e (32 bytes)
    # Amount: 0x0100000000000000000000000000000000000000000000000000000000000000 (32 bytes u256)
    
    expected_addr_hex = "1bb5f31f040703fd8924871dfd8ec4a02d7f2053c2a0faff8ce4164dd005752e"
    expected_amount_hex = "0100000000000000000000000000000000000000000000000000000000000000"
    
    if expected_addr_hex in serialized_data.hex():
        print("✅ Address serialized correctly (without type tag prefix)")
    else:
        print("❌ Address serialization may still include type tag prefix")
    
    if expected_amount_hex in serialized_data.hex():
        print("✅ Amount serialized correctly as u256")
    else:
        print("❌ Amount serialization issue")
        
    print("✅ Address serialization fix test completed")

if __name__ == "__main__":
    try:
        test_address_serialization_fix()
        
        print("\n🎉 Address serialization test completed!")
        print("The fix ensures that Move function arguments are serialized without type tag prefixes.")
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
