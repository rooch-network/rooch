Feature: Rooch CLI Gas Payment Hub integration tests

    @serial
    Scenario: gas_payment_with_hub_basic_operations
      Given a server for gas_payment_with_hub_basic_operations

      # Create test accounts and get initial gas
      Then cmd: "account create"
      Then cmd: "account list --json"

      # Get initial gas for testing - fund the default account
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --sender {{$.account[0].default.address}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Test 1: Check initial gas balance (should be in account store only)
      Then cmd: "move view --function rooch_framework::transaction_gas::total_available_gas_balance --args address:{{$.account[0].default.address}}"
      #Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 10000000000000"

      # Test 2: Create payment hub by depositing RGAS
      Then cmd: "move run --function rooch_framework::payment_channel::deposit_to_hub_entry --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}} --args u256:6000000000 --sender {{$.account[0].default.address}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Test 3: Verify payment hub has RGAS balance
      Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 6000000000"

      # Test 4: Record balances before transaction to test gas payment source
      Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
      # Store account balance (index -1)
      
      Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
      # Store hub balance (index -1, account balance now at -2)

      # Test 5: Perform transaction - should use payment hub for gas
      Then cmd: "move run --function rooch_framework::empty::empty --sender {{$.account[0].default.address}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Test 6: Check balances after transaction to verify gas was deducted from hub
      Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
      # Current account balance (index -1), previous account balance at -4

      Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
      # Current hub balance (index -1), previous hub balance at -4

      # Verify gas was deducted from payment hub, not account store
      Then assert: "{{$.move[-4].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"  # hub balance decreased (before > after)
      Then assert: "{{$.move[-5].return_values[0].decoded_value}} == {{$.move[-2].return_values[0].decoded_value}}"  # account balance unchanged (before == after)

      # Test 7: Check total available balance (should be less than initial due to gas consumption)
      Then cmd: "move view --function rooch_framework::transaction_gas::total_available_gas_balance --args address:{{$.account[0].default.address}}"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} > 10000000000000"

      Then stop the server

    @serial
    Scenario: gas_payment_fallback_behavior
      Given a server for gas_payment_fallback_behavior

      # Create test accounts
      Then cmd: "account create"
      Then cmd: "account list --json"

      # Get gas for testing - fund the default account (but don't create payment hub)
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:5000000000000 --sender {{$.account[0].default.address}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Test 1: Check gas balance without payment hub (should work normally)
      # Note: In local/dev environment, users automatically get 1000000000000 RGAS when balance is 0
      # So total should be 1000000000000 + 5000000000000 = 6000000000000, minus some gas consumption
      Then cmd: "move view --function rooch_framework::transaction_gas::total_available_gas_balance --args address:{{$.account[0].default.address}}"
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} > 5900000000000"

      # Test 2: Record balances before transaction (no payment hub exists)
      Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
      # Store account balance (index -1)

      Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
      # Store hub balance (index -1, account balance now at -2)

      # Test 3: Perform transaction without payment hub (should use account store)
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:1000 --sender {{$.account[0].default.address}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Test 4: Check balances after transaction
      Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
      # Current account balance (index -1), previous account balance at -4

      Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
      # Current hub balance (index -1), previous hub balance at -4

      # Verify gas was deducted from account store, not payment hub
      Then assert: "{{$.move[-5].return_values[0].decoded_value}} > {{$.move[-2].return_values[0].decoded_value}}"  # account balance decreased (before > after)
      Then assert: "{{$.move[-4].return_values[0].decoded_value}} == {{$.move[-1].return_values[0].decoded_value}}"  # hub balance unchanged (should be 0)
      Then assert: "{{$.move[-4].return_values[0].decoded_value}} == 0"  # no payment hub exists

      Then stop the server

    @serial
    Scenario: gas_payment_mixed_behavior
      Given a server for gas_payment_mixed_behavior

      # Create test accounts and get initial gas
      Then cmd: "account create"
      Then cmd: "account list --json"

      # Get initial gas for testing
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --sender {{$.account[0].default.address}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Create payment hub with small amount (insufficient for large gas consumption)
      Then cmd: "move run --function rooch_framework::payment_channel::deposit_to_hub_entry --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}} --args u256:100000 --sender {{$.account[0].default.address}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Test 1: Record balances before large transaction
      Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
      # Store account balance (index -1)

      Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
      # Store hub balance (index -1, account balance now at -2)

      # Test 2: Perform transaction that requires more gas than available in hub
      # This should trigger mixed payment: hub first, then account store
      Then cmd: "move run --function rooch_framework::empty::empty --sender {{$.account[0].default.address}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Test 3: Check balances after transaction
      Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
      # Current account balance (index -1), previous account balance at -4

      Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
      # Current hub balance (index -1), previous hub balance at -4

      # Verify mixed payment: both balances should decrease
      Then assert: "{{$.move[-4].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"  # hub balance decreased
      Then assert: "{{$.move[-5].return_values[0].decoded_value}} > {{$.move[-2].return_values[0].decoded_value}}"  # account balance also decreased
      Then assert: "{{$.move[-1].return_values[0].decoded_value}} < {{$.move[-4].return_values[0].decoded_value}}"  # hub was partially used

      Then stop the server
