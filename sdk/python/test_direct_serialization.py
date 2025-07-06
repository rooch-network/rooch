#!/usr/bin/env python3

import sys
sys.path.append('.')

from rooch.transactions.serializer import TxSerializer
from rooch.transactions.types import FunctionCall, TypeTag, StructTag
from rooch.bcs.serializer import BcsSerializer

# Create a test function call
test_recipient = "0x1e8c6e39a84379ec79dd6722e3d17ac1b95c39f66c6c12672391a5a6607e4a1b"
test_amount = 100

# Create FunctionCall
func_call = FunctionCall(
    function_id="0x3::transfer::transfer_coin",
    type_args=[TypeTag.struct(StructTag(address="0x3", module="gas_coin", name="RGas", type_params=[]))],
    args=[test_recipient, test_amount]
)

print(f"Function call args: {func_call.args}")
print(f"Arg types: {[type(arg) for arg in func_call.args]}")

# Test our serializer
serializer = BcsSerializer()
TxSerializer._encode_function_call(serializer, func_call)

print(f"Serialized function call: {serializer.output().hex()}")
