#!/usr/bin/env python3

# Debug script to test argument serialization

from rooch.bcs.serializer import BcsSerializer
from rooch.address.rooch import RoochAddress

# Test address serialization
test_address = "0x1e8c6e39a84379ec79dd6722e3d17ac1b95c39f66c6c12672391a5a6607e4a1b"
print(f"Testing address: {test_address}")

# Method 1: As string (what we DON'T want)
ser1 = BcsSerializer()
ser1.str(test_address)
print(f"As string: {ser1.output().hex()}")

# Method 2: Using RoochAddress.serialize (what we tried)
ser2 = BcsSerializer()
addr = RoochAddress.from_hex(test_address)
addr.serialize(ser2)
print(f"Using RoochAddress.serialize: {ser2.output().hex()}")

# Method 3: Using fixed_bytes directly (what we want for MoveValue::Address)
ser3 = BcsSerializer()
addr_bytes = addr.to_bytes()
ser3.fixed_bytes(addr_bytes)
print(f"Using fixed_bytes: {ser3.output().hex()}")

# Test integer serialization
test_amount = 100
print(f"\nTesting integer: {test_amount}")

# Method 1: As u64 (what we want)
ser4 = BcsSerializer()
ser4.u64(test_amount)
print(f"As u64: {ser4.output().hex()}")

# Method 2: As u256 (what we don't want for small numbers)
ser5 = BcsSerializer()
ser5.u256(test_amount)
print(f"As u256: {ser5.output().hex()}")

print("\nExpected in transaction hex:")
print(f"Address bytes: {addr_bytes.hex()}")
print(f"Amount as u64: 64000000000000 (little endian)")
