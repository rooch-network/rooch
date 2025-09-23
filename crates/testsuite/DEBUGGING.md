# Enhanced Testsuite Framework Debugging Guide

This document explains the enhanced debugging and error reporting capabilities added to the Rooch integration test framework.

## Environment Variables

The enhanced testsuite framework supports several environment variables for controlling behavior:

### `ROOCH_TEST_LOG_LEVEL`
Controls the verbosity of test output.

- `minimal`: Only show failures and final results
- `normal`: Default behavior with basic logging (default)
- `verbose`: Include template context changes and detailed information
- `debug`: Full execution trace with all debugging information

Example:
```bash
ROOCH_TEST_LOG_LEVEL=verbose cargo test --test integration
```

### `ROOCH_TEST_TEMPLATE_DEBUG`
Enable detailed template variable resolution debugging.

- `true`: Show template resolution steps, available variables, and usage
- `false`: Normal template resolution (default)

Example:
```bash
ROOCH_TEST_TEMPLATE_DEBUG=true cargo test --test integration
```

### `ROOCH_TEST_SHOW_PROGRESS`
Show progressive test execution indicators.

- `true`: Display step-by-step progress with emoji indicators
- `false`: No progress indicators (default)

Example:
```bash
ROOCH_TEST_SHOW_PROGRESS=true cargo test --test integration
```

### `ROOCH_TEST_TIMEOUT`
Set command execution timeout in seconds (default: 30).

Example:
```bash
ROOCH_TEST_TIMEOUT=60 cargo test --test integration
```

## Enhanced Error Messages

### Before (Current)
```
ERROR integration: run_cli cmd: payment-channel fail: String("Transaction error: Transaction execution failed...")
thread 'payment_channel_operations' panicked at crates/testsuite/tests/integration.rs:554:5:
splited_args should not empty, the orginal_args:\
```

### After (Enhanced)
```
🧪 [15/42] payment_channel_operations - Running command: payment-channel query-revenue --owner {{$.did[1].did_address}}
  ✅ payment-channel query-revenue --owner {{$.did[1].did_address}}

🧪 [16/42] payment_channel_operations - Running command: payment-channel withdraw-revenue --owner {{$.did[1].did_address}} --amount 5000 --sender {{$.did[1].did_address}}
  ❌ payment-channel withdraw-revenue --owner {{$.did[1].did_address}} --amount 5000 --sender {{$.did[1].did_address}}
     Error: Transaction execution failed: MoveAbort { location: 0x3::payment_revenue, abort_code: 1 }
     Template vars used: ["$.did[1].did_address"]
     Available context keys: ["account", "did", "payment-channel", "address_mapping"]

❌ Assertion failed:
   Expression: executed == moveabort
   Expected: executed
   Actual: moveabort
   Operator: ==
   Template vars used: ["$.payment-channel[-1].execution_info.status.type"]
```

## Template Debugging

When `ROOCH_TEST_TEMPLATE_DEBUG=true` is set, failed template resolution shows detailed information:

### Template Resolution Failure
```
❌ Template resolution failed:
   Expression: {{$.payment-channel[-1].execution_info.status.type}} == executed
   Available vars: ["account", "did", "payment-channel", "address_mapping"]
   Used vars: ["$.payment-channel[-1].execution_info.status.type"]
   Resolution steps: [
     "Found template variable: $.payment-channel[-1].execution_info.status.type",
     "Template resolution completed"
   ]
```

### Assertion Parsing Errors
```
❌ Failed to parse assertion arguments:
   Original: {{$.invalid.template}} == test
   After template resolution: 
   Parse error: Template variable resolution failed
```

## Progressive Test Execution

When `ROOCH_TEST_SHOW_PROGRESS=true` is set, you see clear progress indicators:

```
🧪 [1/42] payment_channel_operations - Starting server
🧪 [2/42] payment_channel_operations - Running command: account create
  ✅ account create
🧪 [3/42] payment_channel_operations - Running command: account list --json
  ✅ account list --json
🧪 [4/42] payment_channel_operations - Running command: move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --json --sender {{$.account[0].default.address}}
  ✅ move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --json --sender rooch1dqk9pjqgpj9ddg6hl8lp3q9xgje8...
```

## Common Debugging Patterns

### 1. Template Variable Issues
When you see template resolution errors, check:
- Variable names for typos
- Array indices (remember -1 is the last element)
- Available context keys in the error message

### 2. Assertion Failures
Enhanced assertion messages show:
- The exact values being compared
- Which template variables were used
- Available context for debugging

### 3. Command Execution Failures
Enhanced command error messages include:
- Full command that failed
- Template variables that were resolved
- Available context keys
- Clear error message from the underlying command

## Running with Enhanced Debugging

### Full Debugging Mode
```bash
ROOCH_TEST_LOG_LEVEL=debug \
ROOCH_TEST_TEMPLATE_DEBUG=true \
ROOCH_TEST_SHOW_PROGRESS=true \
ROOCH_TEST_TIMEOUT=60 \
cargo test --test integration -- --nocapture
```

### Minimal Output for CI
```bash
ROOCH_TEST_LOG_LEVEL=minimal \
cargo test --test integration
```

### Development Mode
```bash
ROOCH_TEST_LOG_LEVEL=verbose \
ROOCH_TEST_SHOW_PROGRESS=true \
cargo test --test integration -- --nocapture
```

## Backward Compatibility

All existing `.feature` files continue to work without any changes. The enhancements are opt-in through environment variables and provide additional debugging information when needed.