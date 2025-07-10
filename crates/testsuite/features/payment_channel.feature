Feature: Rooch CLI Payment Channel integration tests

    @serial
    Scenario: payment_channel_operations
      Given a server for payment_channel_operations

      # Create test accounts and get gas first
      Then cmd: "account create"
      Then cmd: "account create" 
      Then cmd: "account list --json"

      # Get gas for testing - fund the first account
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Create DID for sender (required for payment channels) 
      Then cmd: "did create self"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

      # Test 1: Initialize payment hub with deposit
      Then cmd: "payment-channel init --amount 1000000000"

      # Test 2: Open payment channel with sub-channels (use same account as receiver for testing)
      Then cmd: "payment-channel open --receiver {{$.account[0].default.hex_address}} --vm-id-fragments account-key"

      # Test 3: Create RAV (Receipt and Voucher) for off-chain payment
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[-1].channel_id}} --vm-id-fragment account-key --amount 10000 --nonce 1"

      # Test 4: Query channel information  
      Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[-2].channel_id}}"

      # Test 5: Query hub ID for sender
      Then cmd: "payment-channel query hub-id --address {{$.account[0].default.hex_address}}"

      # Test 6: Query active channel count for sender
      Then cmd: "payment-channel query active-count --address {{$.account[0].default.hex_address}}"

      # Test 7: Calculate channel ID for verification
      Then cmd: "payment-channel query calc-channel-id --sender {{$.account[0].default.hex_address}} --receiver {{$.account[0].default.hex_address}}"

      # Test 8: Query sub-channel information
      Then cmd: "payment-channel query sub-channel --channel-id {{$.payment-channel[-6].channel_id}} --vm-id-fragment account-key"

      # Test 9: Cancel the channel to test cancellation workflow
      Then cmd: "payment-channel cancel --channel-id {{$.payment-channel[-7].channel_id}}"

      # Test 10: Query cancellation information
      Then cmd: "payment-channel query cancellation --channel-id {{$.payment-channel[-8].channel_id}}"

      Then stop the server

    @serial
    Scenario: payment_channel_error_cases
      Given a server for payment_channel_error_cases

      # Create test accounts
      Then cmd: "account create"
      Then cmd: "account list --json"

      # Get gas for testing
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Create DID for sender
      Then cmd: "did create self"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

      # Test 1: Try to open channel without initializing hub first (should fail)
      # Note: This might succeed if hub creation is automatic, but it's a good test case
      Then cmd: "payment-channel open --receiver {{$.account[0].default.hex_address}} --vm-id-fragments test-key"

      # Test 2: Initialize payment hub
      Then cmd: "payment-channel init --amount 1000000000"

      # Test 3: Try to open channel to self (should fail)
      Then cmd: "payment-channel open --receiver {{$.account[0].default.hex_address}} --vm-id-fragments self-key"
      # Note: Based on code, this should fail with "Sender and receiver cannot be the same address"

      # Test 4: Try to open channel with empty VM ID fragments (should fail) 
      Then cmd: "payment-channel open --receiver 0x1234 --vm-id-fragments ''"
      # Note: This should fail with "At least one VM ID fragment is required"

      # Test 5: Try to create RAV with invalid channel ID
      Then cmd: "payment-channel create-rav --channel-id 0x0000000000000000000000000000000000000000000000000000000000000000 --vm-id-fragment test-key --amount 1000 --nonce 1"

      Then stop the server

    @serial
    Scenario: payment_channel_multi_subchannel
      Given a server for payment_channel_multi_subchannel

      # Create test accounts
      Then cmd: "account create"
      Then cmd: "account list --json"

      # Get gas for testing
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Create DID for sender
      Then cmd: "did create self"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

      # Test: Initialize hub and open channel with multiple sub-channels
      Then cmd: "payment-channel init --amount 2000000000"

      # Open channel with multiple VM ID fragments (use self as receiver for testing)
      Then cmd: "payment-channel open --receiver {{$.account[0].default.hex_address}} --vm-id-fragments key-1,key-2,key-3"

      # Create RAVs for different sub-channels
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[-1].channel_id}} --vm-id-fragment key-1 --amount 50000 --nonce 1"
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[-2].channel_id}} --vm-id-fragment key-2 --amount 30000 --nonce 1"
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[-3].channel_id}} --vm-id-fragment key-3 --amount 20000 --nonce 1"

      # Query individual sub-channels
      Then cmd: "payment-channel query sub-channel --channel-id {{$.payment-channel[-6].channel_id}} --vm-id-fragment key-1"
      Then cmd: "payment-channel query sub-channel --channel-id {{$.payment-channel[-7].channel_id}} --vm-id-fragment key-2"
      Then cmd: "payment-channel query sub-channel --channel-id {{$.payment-channel[-8].channel_id}} --vm-id-fragment key-3"

      # Test cancellation with multiple sub-channels
      Then cmd: "payment-channel cancel --channel-id {{$.payment-channel[-9].channel_id}}"

      Then stop the server 