Feature: Rooch CLI Payment Channel integration tests

    @serial
    Scenario: payment_channel_operations
      Given a server for payment_channel_operations

      # Create test accounts and get gas first
      Then cmd: "account create"
      Then cmd: "account list --json"

      # Get gas for testing - fund the default account (sender)
      Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --json --sender {{$.account[0].default.address}}"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Create DID for sender (required for payment channels) 
      Then cmd: "did create self"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

      # Test 1: Initialize payment hub with deposit
      Then cmd: "payment-channel init --owner {{$.did[0].did}} --amount 1000000000"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"


      # Test 2: Open payment channel with sub-channels (use second account as receiver)
      Then cmd: "payment-channel open --sender {{$.did[0].did}} --receiver {{$.account[0].account0.address}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Test 4: Query channel information  
      Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[-1].channel_id}} --list-sub-channels"
      Then assert: "{{$.payment-channel[-1].sub_channels_count}} == 1"

      # Test 3: Create RAV (Receipt and Voucher) for off-chain payment
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[-1].channel_id}} --vm-id-fragment account-key --amount 10000 --nonce 1 --sender {{$.did[0].did}}"
      Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 1"

      Then cmd: "payment-channel claim --rav {{$.payment-channel[-1].encoded}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status}} == executed"

      # Test 5: Query hub information for receiver
      Then cmd: "payment-channel query hub --owner {{$.account[0].account0.address}}"
      Then assert: "{{$.payment-channel[-1].balances[0].amount}} == 10000"

      # Test 6: Query sub-channel information
      Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[-3].channel_id}} --vm-id account-key"

      # Test 7: Cancel the channel to test cancellation workflow
      Then cmd: "payment-channel cancel --channel-id {{$.payment-channel[-4].channel_id}}"

      Then stop the server


    @serial
    Scenario: payment_channel_multi_subchannel
      Given a server for payment_channel_multi_subchannel

      # Create test accounts
      Then cmd: "account create"
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

      # Open channel with multiple VM ID fragments (use second account as receiver)
      Then cmd: "payment-channel open --receiver {{$.account[1].default.hex_address}} --vm-id-fragments key-1,key-2,key-3"

      # Create RAVs for different sub-channels, specify sender DID address
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[-1].channel_id}} --vm-id-fragment key-1 --amount 50000 --nonce 1 --sender {{$.account[0].default.hex_address}}"
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[-2].channel_id}} --vm-id-fragment key-2 --amount 30000 --nonce 1 --sender {{$.account[0].default.hex_address}}"
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[-3].channel_id}} --vm-id-fragment key-3 --amount 20000 --nonce 1 --sender {{$.account[0].default.hex_address}}"

      # Query individual sub-channels
      Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[-6].channel_id}} --vm-id key-1"
      Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[-7].channel_id}} --vm-id key-2"
      Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[-8].channel_id}} --vm-id key-3"

      # Test cancellation with multiple sub-channels
      Then cmd: "payment-channel cancel --channel-id {{$.payment-channel[-9].channel_id}}"

      Then stop the server 