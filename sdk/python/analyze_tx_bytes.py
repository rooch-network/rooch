#!/usr/bin/env python3

"""
Detailed comparison of transaction serialization
"""

import sys
import os

# Add the project root to the Python path for imports
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

def analyze_transaction_bytes():
    """Analyze the byte-level differences in transaction serialization"""
    
    print("Analyzing transaction bytes...")
    
    # Correct transaction args from the successful transaction
    correct_addr = "0x1bb5f31f040703fd8924871dfd8ec4a02d7f2053c2a0faff8ce4164dd005752e"
    correct_amount = "0x0100000000000000000000000000000000000000000000000000000000000000"
    
    print(f"Correct address: {correct_addr}")
    print(f"Correct amount: {correct_amount}")
    print(f"Address length: {(len(correct_addr) - 2) // 2} bytes")  # Remove 0x prefix and divide by 2
    print(f"Amount length: {(len(correct_amount) - 2) // 2} bytes")
    
    # Test our serialization
    from rooch.transactions.move.move_types import TransactionArgument
    from rooch.transactions.tags.type_tags import TypeTagCode
    from rooch.bcs.serializer import BcsSerializer
    
    # Create arguments
    addr_arg = TransactionArgument(TypeTagCode.ADDRESS, correct_addr)
    amount_arg = TransactionArgument(TypeTagCode.U256, 1)
    
    print(f"\n=== Our Address Argument ===")
    print(f"Type: {addr_arg.type_tag}")
    print(f"Value: {addr_arg.value}")
    
    # Test value-only serialization
    addr_ser = BcsSerializer()
    addr_arg.serialize_value_only(addr_ser)
    addr_bytes = addr_ser.output()
    print(f"Value-only bytes: {addr_bytes.hex()}")
    print(f"Length: {len(addr_bytes)} bytes")
    
    print(f"\n=== Our Amount Argument ===")
    print(f"Type: {amount_arg.type_tag}")
    print(f"Value: {amount_arg.value}")
    
    # Test value-only serialization
    amount_ser = BcsSerializer()
    amount_arg.serialize_value_only(amount_ser)
    amount_bytes = amount_ser.output()
    print(f"Value-only bytes: {amount_bytes.hex()}")
    print(f"Length: {len(amount_bytes)} bytes")
    
    # Compare with expected
    expected_addr_bytes = bytes.fromhex(correct_addr[2:])  # Remove 0x prefix
    expected_amount_bytes = bytes.fromhex(correct_amount[2:])  # Remove 0x prefix
    
    print(f"\n=== Comparison ===")
    print(f"Expected address bytes: {expected_addr_bytes.hex()}")
    print(f"Our address bytes:      {addr_bytes.hex()}")
    print(f"Address match: {addr_bytes == expected_addr_bytes}")
    
    print(f"Expected amount bytes: {expected_amount_bytes.hex()}")
    print(f"Our amount bytes:      {amount_bytes.hex()}")
    print(f"Amount match: {amount_bytes == expected_amount_bytes}")
    
    # Test sequence serialization (this is what happens in FunctionArgument)
    print(f"\n=== Sequence Serialization Test ===")
    args = [addr_arg, amount_arg]
    
    seq_ser = BcsSerializer()
    seq_ser.sequence(args, lambda s, item: item.serialize_value_only(s))
    seq_bytes = seq_ser.output()
    print(f"Sequence bytes: {seq_bytes.hex()}")
    
    # Check if this has length prefixes
    print(f"Sequence length: {len(seq_bytes)} bytes")
    
    # Manual breakdown of what we expect:
    # - 2 items in sequence -> 1 byte length prefix (0x02)
    # - Address: 32 bytes (no length prefix for fixed_bytes)
    # - Amount: 32 bytes (no length prefix for u256)
    # Total expected: 1 + 32 + 32 = 65 bytes
    
    expected_total = 1 + 32 + 32
    print(f"Expected sequence total: {expected_total} bytes")
    print(f"Actual sequence total: {len(seq_bytes)} bytes")
    
    if len(seq_bytes) > expected_total:
        extra_bytes = len(seq_bytes) - expected_total
        print(f"❌ We have {extra_bytes} extra bytes - likely length prefixes!")
    elif len(seq_bytes) == expected_total:
        print(f"✅ Sequence length looks correct")
    else:
        print(f"❌ Sequence is shorter than expected")

if __name__ == "__main__":
    try:
        analyze_transaction_bytes()
    except Exception as e:
        print(f"❌ Analysis failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
