# Payment Channel Command Usage Guide

## Optimized Commands

### Close Channel Command

The `close` command now supports two input methods for better user experience:

#### Method 1: Using Multibase Encoded RAVs (Recommended)

```bash
# Close channel with multiple RAV proofs
rooch payment-channel close \
    --channel-id 0x123... \
    --ravs "u1ABC..." \
    --ravs "u1DEF..." \
    --ravs "u1GHI..."
```

#### Method 2: Legacy Hex Proofs (Backward Compatible)

```bash
# Close channel with hex-encoded proofs
rooch payment-channel close \
    --channel-id 0x123... \
    --proofs "deadbeef..."
```

### Cancel Channel Command

The `cancel` command now supports optional proof submission:

#### Method 1: Simple Cancellation (No Proof)

```bash
# Simple cancellation without proof
rooch payment-channel cancel \
    --channel-id 0x123...
```

#### Method 2: Cancellation with RAV Proof

```bash
# Cancellation with multibase encoded RAV proof
rooch payment-channel cancel \
    --channel-id 0x123... \
    --rav "u1ABC..."
```

#### Method 3: Cancellation with Individual Parameters

```bash
# Cancellation with individual proof parameters
rooch payment-channel cancel \
    --channel-id 0x123... \
    --individual-params \
    --vm-id-fragment "key-1" \
    --amount 1000 \
    --nonce 5 \
    --signature "deadbeef..."
```

## Workflow Integration

### Complete Workflow Example

```bash
# 1. Create a RAV
rooch payment-channel create-rav \
    --channel-id 0x123... \
    --vm-id-fragment "key-1" \
    --amount 1000 \
    --nonce 5 \
    --sender 0xabc...

# Output includes an 'encoded' field with multibase string like "u1ABC..."

# 2. Use the encoded RAV for closing
rooch payment-channel close \
    --channel-id 0x123... \
    --ravs "u1ABC..."

# OR use it for cancellation
rooch payment-channel cancel \
    --channel-id 0x123... \
    --rav "u1ABC..."
```

## Benefits

1. **Improved User Experience**: No need to manually construct hex-encoded proofs
2. **Type Safety**: Automatic validation of RAV data structure
3. **Backward Compatibility**: Legacy hex proofs still supported
4. **Flexibility**: Multiple input methods for different use cases
5. **Error Prevention**: Channel ID validation prevents mismatched proofs

## Migration Guide

If you're currently using hex-encoded proofs, you can:

1. **Keep using legacy method**: Use `--proofs` flag with hex data
2. **Migrate to RAV method**: Use `create-rav` command followed by `--ravs` flag
3. **Use hybrid approach**: Mix both methods as needed during transition 