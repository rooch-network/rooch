#!/usr/bin/env python3

"""
Test script to verify u256 type fixing for transfer_coin function
"""

import sys
import os

# Add the project root to the Python path for imports
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

def test_u256_parameter_serialization():
    """Test that integer parameters are now serialized as u256 by default"""
    
    print("Testing u256 parameter type inference...")
    
    # Test the updated FunctionArgument logic
    from rooch.transactions.move.move_types import FunctionArgument
    from rooch.transactions.tags.type_tags import TypeTag, StructTag, TypeTagCode
    
    # Create a test transfer amount
    test_amount = 1000000000  # 1 RGAS (with 8 decimal places)
    test_recipient = "0x1234567890abcdef1234567890abcdef12345678"
    
    # Build function argument - this should now use u256 for integers
    function_id = "0x3::transfer::transfer_coin"
    ty_args = [TypeTag.struct(StructTag(
        address="0x3", 
        module="gas_coin", 
        name="RGas", 
        type_params=[]
    ))]
    args = [test_recipient, test_amount]  # These should be inferred as ADDRESS and U256
    
    func_arg = FunctionArgument(function_id, ty_args, args)
    
    # Check the inferred types
    print(f"Arguments created: {len(func_arg.args)}")
    for i, arg in enumerate(func_arg.args):
        print(f"Arg {i}: type={arg.type_tag}, value={arg.value}")
        if i == 0:  # recipient address
            assert arg.type_tag == TypeTagCode.ADDRESS, f"Expected ADDRESS, got {arg.type_tag}"
            print("✅ Address parameter correctly inferred as ADDRESS")
        elif i == 1:  # amount
            assert arg.type_tag == TypeTagCode.U256, f"Expected U256, got {arg.type_tag}"
            print("✅ Integer parameter correctly inferred as U256")
    
    print("✅ Type inference test passed - integers now default to U256")
    
    # Test serialization
    from rooch.bcs.serializer import BcsSerializer
    serializer = BcsSerializer()
    func_arg.serialize(serializer)
    serialized_data = serializer.output()
    
    print(f"Serialized function argument: {serialized_data.hex()}")
    print("✅ Serialization test completed")

if __name__ == "__main__":
    try:
        # Test type inference
        test_u256_parameter_serialization()
        
        print("\n🎉 U256 type inference test completed successfully!")
        print("The fix ensures that integer parameters default to U256 instead of U64,")
        print("which is compatible with Rooch framework functions like transfer_coin.")
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
