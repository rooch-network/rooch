# Rooch Integration Tests (BDD/Cucumber) Guide

This document targets **Rust / Move** developers and AI assistants. It explains the Cucumber-based integration test framework located in `crates/testsuite`, its syntax, and best practices.

> All commands assume you are at the repository root and have `cargo`, `make`, and other development dependencies installed.

---

## 1. Directory Layout

```
crates/
  testsuite/
    features/              # *.feature files written in Gherkin
      payment_channel.feature   # Payment-channel tests
      cmd.feature               # Comprehensive CLI tests (528 lines)
      ...
    tests/
      integration.rs            # Step definitions & test entry point
```

* `*.feature` files describe test scenarios in **Gherkin**.
* `integration.rs` maps **Given / Then** steps to Rust code via [`cucumber-rs`](https://github.com/cucumber-rs/cucumber).

## 2. Running the Tests

1. **Run all integration tests**

   ```bash
   # Single-thread to avoid port/resource conflicts
   cargo test --test integration -- --test-threads=1
   ```

2. **Via Makefile (recommended)** [[repo rules §3.1]]

   ```bash
   make test-integration
   ```

---

## 3. Gherkin Basics

```gherkin
Feature: <module description>
  @serial                     # Run scenario serially (avoid parallel servers)
  Scenario: <scenario name>
    Given a server for <id>
    Then cmd: "account create"
    Then assert: "{{$.account[-1]}} != null"
    Then stop the server
```

* **Feature / Scenario** are standard Gherkin constructs.
* The **@serial** tag forces Cucumber to execute the scenario serially, preventing port collisions.
* **Given / Then** steps are matched by macros in `integration.rs`.

---

## 4. Step Reference

| Type | Example | Description |
|------|---------|-------------|
| Given | `Given a server for payment_channel` | Start a Rooch full-node server on a random local port. |
| Given | `Given a bitcoind server for btc` | Start a regtest Bitcoin node via [testcontainers](https://github.com/testcontainers/testcontainers-rs). |
| Given | `Given a ord server for inscriptions` | Start an Ord server (depends on bitcoind). |
| Then  | `Then stop the server` | Shut down the Rooch server. |
| Then  | `Then stop the bitcoind server` | Stop the bitcoind container. |
| Then  | `Then stop the ord server` | Stop the Ord container. |
| Then  | `Then sleep: "5"` | Sleep **N** seconds. |
| Then  | `Then cmd: "<cli args>"` | Execute `rooch <cli args>`; the framework auto-injects `--config-dir`. |
| Then  | `Then cmd bitcoin-cli: "<args>"` | Run `bitcoin-cli -regtest <args>` inside the bitcoind container. |
| Then  | `Then cmd ord: "<args>"` | Run `ord --regtest ... <args>` inside the Ord container. |
| Then  | `Then cmd ord bash: "<bash>"` | Execute an arbitrary bash command inside Ord container. |
| Then  | `Then cmd bitseed: "<args>"` | Launch a one-shot Bitseed CLI container. |
| Then  | `Then assert: "<expr>"` | Evaluate assertions (see §6). |

> **Hint:** Every `cmd:` step tries to parse **stdout** as JSON and stores the result in a [`TemplateContext`](https://docs.rs/jpst). The key equals the first word of the command, e.g., `account`, `rpc`, `payment-channel`.

---

## 5. TemplateContext & Placeholders

`integration.rs` writes each step result into a [`jpst::TemplateContext`](https://docs.rs/jpst). Placeholders wrapped by `{{ ... }}` are resolved **before** the step is executed.

### 5.1 Basic Syntax

| Syntax | Meaning |
|--------|---------|
| `{{$.<key>[<idx>]}}` | Fetch element `<idx>` from the result list of `<key>`. `<idx>` supports positive and negative indices (`-1` = last result). |
| `{{$.<key>[<idx>].<field>}}` | Dive into nested **JSON** using dot notation. |
| `{{$.<key>[<idx>]["<field with space>"]}}` | Use `"..."` if the field contains spaces or special chars. |
| `{{$.address_mapping.<alias>}}` | Special key populated from `WalletContext` during initialization. |

> `$` is always the root; the first level key is the **command name** (without prefix). Results are stored chronologically.

### 5.2 Index Rules

* **Positive**: counted from the beginning (`0` = first call).
* **Negative**: counted from the end (`-1` = last call, `-2` = second-to-last).

```gherkin
{{$.rpc[0]}}       # First rpc result
{{$.rpc[-1]}}      # Most recent rpc result
{{$.account[-3]}}  # Third-to-last account result
```

### 5.3 Deep JSON Access

* Nested field: `{{$.payment-channel[-1].execution_info.status.type}}`
* Array element: `{{$.payment-channel[-1].balances[0].amount}}`
* Dynamic key:
  ```gherkin
  # field contains '<', '>' etc.
  {{$.rpc[-1]["object_type"]}}
  ```

### 5.4 Escaping & Spaces

* Placeholders **must not** contain unmatched braces or quotes.
* When the operand needs surrounding quotes/spaces, add them **outside** the placeholder:
  ```gherkin
  Then assert: "'{{$.rpc[-1].balance}}' != '0'"
  ```

### 5.5 Combined Examples

```gherkin
# Use the latest account address as sender
--sender {{$.account[-1].account0.address}}

# Reuse channel_id from the previous payment-channel call
auto_var={{$.payment-channel[1].channel_id}}

# Compare two values from different results
Then assert: "{{$.rpc[-1][0].value}} == {{$.rpc[-2][0].value}}"
```

> If a placeholder fails to resolve (missing key / out-of-range index), `jpst` panics and the test stops.

---

## 6. Assertion Expressions

Example:

```gherkin
Then assert: "'{{$.rpc[-1].balance}}' != '0'"
```

### 6.1 Supported Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equality comparison | `"{{$.balance}} == 1000"` |
| `!=` | Inequality comparison | `"{{$.status}} != error"` |
| `contains` | String contains (case-sensitive) | `"{{$.message}} contains success"` |
| `not_contains` | String does not contain (case-insensitive fallback) | `"{{$.error}} not_contains timeout"` |
| `>` | Greater than (numeric) | `"{{$.balance}} > 0"` |
| `<` | Less than (numeric) | `"{{$.gas_used}} < 1000000"` |
| `>=` | Greater than or equal (numeric) | `"{{$.amount}} >= 100"` |
| `<=` | Less than or equal (numeric) | `"{{$.fee}} <= 50000"` |

### 6.2 Numeric Comparison Precision

The framework supports **high-precision numeric comparisons** without precision loss:

- **u128/i128**: Blockchain amounts (up to 128-bit integers) are compared directly without conversion to floating-point
- **f64**: Floating-point numbers are supported with proper NaN handling
- **Auto-detection**: The system automatically detects the appropriate numeric type

```gherkin
# Large blockchain amounts (no precision loss)
Then assert: "{{$.hub_balance}} > {{$.account_balance}}"

# Floating-point comparisons
Then assert: "{{$.exchange_rate}} >= 1.5"
```

### 6.3 Multiple Assertions

Multiple assertions can be written on **one line** — every three tokens form a rule:

```gherkin
Then assert: "{{$.a}} == 1 {{$.b}} != 2"
```

Wrap operands in quotes to protect spaces.

---

## 7. Writing a New Feature

1. Create `crates/testsuite/features/<name>.feature`.
2. Add `@serial` if the test must run exclusively.
3. **Start the server**:
   ```gherkin
   Given a server for <id>
   ```
4. Optionally start bitcoind / Ord.
5. **Run CLI / RPC commands**. Example: request gas
   ```gherkin
   Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000 --json"
   Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
   ```
6. **Assert business logic** using placeholders.
7. **Stop the server**:
   ```gherkin
   Then stop the server
   ```

---

## 8. payment_channel.feature Walk-through

```gherkin
Scenario: payment_channel_operations
  # 1) Start server
  Given a server for payment_channel_operations

  # 2) Create accounts & get gas
  Then cmd: "account create"
  ...

  # 3) Initialize hub
  Then cmd: "payment-channel init --owner {{$.did[0].did}} --amount 1000000000"

  # 4) Open sub-channel & query
  Then cmd: "payment-channel open --sender ..."
  Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[1].channel_id}} --list-sub-channels"

  # 5) Create & claim RAV
  Then cmd: "payment-channel create-rav ..."
  Then cmd: "payment-channel claim --rav {{$.payment-channel[-1].encoded}}"

  # 6) Cancel & finalize
  Then cmd: "payment-channel cancel --channel-id ..."
  Then cmd: "move run --function 0x3::timestamp::fast_forward_seconds_for_local --args u64:86401"
  Then cmd: "payment-channel finalize-cancellation --channel-id ..."

  Then stop the server
```

* `TemplateContext` transfers variables such as `channel_id` and `signed_rav` across steps.
* Demonstrates **time-travel** (timestamp module) and the complete payment-channel lifecycle.

---

## 9. Advanced Testing Patterns & Best Practices

### 9.1 Command History Indexing

Understanding the `move[-N]` indexing pattern is crucial for complex test scenarios:

```gherkin
# Record balances before transaction
Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"

# Execute transaction
Then cmd: "move run --function rooch_framework::empty::empty --sender {{$.account[0].default.address}} --json"

# Check balances after transaction
Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"

# Compare before/after values using correct indices
Then assert: "{{$.move[-4].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"  # hub balance decreased
Then assert: "{{$.move[-5].return_values[0].decoded_value}} == {{$.move[-2].return_values[0].decoded_value}}"  # account balance unchanged
```

**Key insight**: `move[-1]` refers to the **last** move command in chronological order, `move[-2]` to the second-to-last, etc.

### 9.2 Testing Gas Payment Sources

When testing gas payment mechanisms, consider the **automatic gas allocation** in local/dev environments:

```gherkin
# In local/dev, users automatically receive 1000000000000 RGAS when balance is 0
# So faucet_entry(5000000000000) results in ~6000000000000 total (minus gas consumption)
Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:5000000000000 --sender {{$.account[0].default.address}} --json"
Then cmd: "move view --function rooch_framework::transaction_gas::total_available_gas_balance --args address:{{$.account[0].default.address}}"
Then assert: "{{$.move[-1].return_values[0].decoded_value}} > 5900000000000"  # Account for automatic allocation + gas consumption
```

### 9.3 Precise Balance Verification Strategy

To verify gas payment sources (hub vs account store), use this pattern:

1. **Record before-state**: Capture both account and hub balances
2. **Execute transaction**: Perform the operation that consumes gas
3. **Record after-state**: Capture balances again
4. **Assert changes**: Use numeric comparisons to verify the expected source was used

```gherkin
# Scenario: Verify gas comes from payment hub, not account store
Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
Then cmd: "move run --function rooch_framework::empty::empty --sender {{$.account[0].default.address}} --json"
Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"

# Verify hub balance decreased (gas was deducted from hub)
Then assert: "{{$.move[-4].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"
# Verify account balance unchanged (gas was NOT deducted from account)
Then assert: "{{$.move[-5].return_values[0].decoded_value}} == {{$.move[-2].return_values[0].decoded_value}}"
```

### 9.4 Type Arguments in CLI Commands

When calling generic Move functions, use the correct syntax for type arguments:

```gherkin
# Correct: Use --type-args for generic type parameters
Then cmd: "move run --function rooch_framework::payment_channel::deposit_to_hub_entry --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}} --args u256:6000000000 --sender {{$.account[0].default.address}} --json"

# Correct: For view functions too
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
```

### 9.5 Testing Mixed Payment Scenarios

For complex scenarios like mixed gas payment (hub + account store), create dedicated test cases:

```gherkin
Scenario: gas_payment_mixed_behavior
  # Setup: Create payment hub with insufficient balance for full gas payment
  Then cmd: "move run --function rooch_framework::payment_channel::deposit_to_hub_entry --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}} --args u256:100000 --sender {{$.account[0].default.address}} --json"
  
  # Execute transaction requiring more gas than hub balance
  # This should trigger: hub balance → 0, remaining gas from account store
  Then cmd: "move run --function rooch_framework::empty::empty --sender {{$.account[0].default.address}} --json"
  
  # Verify both sources were used
  Then assert: "{{$.move[-4].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"  # hub decreased
  Then assert: "{{$.move[-5].return_values[0].decoded_value}} > {{$.move[-2].return_values[0].decoded_value}}"  # account also decreased
```

---

## 10. Debugging Tips

1. **Inspect context**: On failure, the framework prints the entire `TemplateContext`.
2. **Port busy**: `integration.rs` waits for the port to free; after 60 s it panics.
3. **JSON parsing failed**: Ensure the CLI command includes `--json`; otherwise stdout is stored as plain string.
4. **Index calculation errors**: When assertions fail with unexpected values, double-check your `move[-N]` indices by counting commands chronologically.
5. **Precision issues**: If numeric comparisons fail unexpectedly, verify that large integers aren't being truncated. The framework now supports full u128 precision.
6. **Template resolution failures**: Use descriptive comments to document what each `{{$.move[-N]}}` reference should contain.
7. **Gas consumption variations**: In local/dev environments, account for automatic RGAS allocation (1000000000000) when testing initial balances.

### 10.1 Enhanced Debugging Features (New!)

The testsuite framework now includes enhanced debugging capabilities controlled by environment variables:

#### Environment Variables
- `ROOCH_TEST_LOG_LEVEL=minimal|normal|verbose|debug` - Control output verbosity
- `ROOCH_TEST_TEMPLATE_DEBUG=true` - Enable detailed template resolution debugging
- `ROOCH_TEST_SHOW_PROGRESS=true` - Show progressive test execution indicators
- `ROOCH_TEST_TIMEOUT=60` - Set command timeout in seconds (default: 30)
- `ROOCH_TEST_HELP=true` - Display environment variable help

#### Example Usage
```bash
# Full debugging mode
ROOCH_TEST_LOG_LEVEL=debug \
ROOCH_TEST_TEMPLATE_DEBUG=true \
ROOCH_TEST_SHOW_PROGRESS=true \
cargo test --test integration -- --nocapture

# Development mode with progress indicators
ROOCH_TEST_LOG_LEVEL=verbose \
ROOCH_TEST_SHOW_PROGRESS=true \
cargo test --test integration -- --nocapture

# Minimal output for CI
ROOCH_TEST_LOG_LEVEL=minimal \
cargo test --test integration
```

#### Enhanced Error Messages

**Before:**
```
ERROR integration: run_cli cmd: payment-channel fail: String("Transaction error...")
thread 'test' panicked at crates/testsuite/tests/integration.rs:554:5:
splited_args should not empty, the orginal_args:\
```

**After:**
```
🧪 [16/42] payment_channel_operations - Running command: payment-channel withdraw-revenue
  ❌ payment-channel withdraw-revenue --owner rooch1abc... --amount 5000
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

For detailed documentation, see: `crates/testsuite/DEBUGGING.md`

### 10.2 Common Assertion Patterns

```gherkin
# Verify transaction succeeded
Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

# Check balance decreased (gas consumed)
Then assert: "{{$.move[-2].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"

# Verify balance unchanged
Then assert: "{{$.move[-2].return_values[0].decoded_value}} == {{$.move[-1].return_values[0].decoded_value}}"

# Check non-zero balance
Then assert: "{{$.move[-1].return_values[0].decoded_value}} > 0"
```

---

## 11. References

* Cucumber for Rust: <https://github.com/cucumber-rs/cucumber>
* jpst template engine: <https://crates.io/crates/jpst>
* Rooch developer docs: `docs/dev-guide/`

---

Happy testing! Pull requests are welcome 😄 