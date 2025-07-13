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
      Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[1].channel_id}} --list-sub-channels"
      Then assert: "{{$.payment-channel[-1].sub_channels_count}} == 1"

      # Test 3: Create RAV (Receipt and Voucher) for off-chain payment
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 10000 --nonce 1 --sender {{$.did[0].did}}"
      Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 1"

      Then cmd: "payment-channel claim --rav {{$.payment-channel[-1].encoded}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Test 5: Query hub information for receiver
      Then cmd: "payment-channel query hub --owner {{$.account[0].account0.address}}"
      Then assert: "{{$.payment-channel[-1].balances[0].amount}} == 10000"


      # Test 7: Cancel the channel to test cancellation workflow
      Then cmd: "payment-channel cancel --channel-id {{$.payment-channel[1].channel_id}} --sender {{$.did[0].did}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Test 8: Dispute (challenge) the cancellation with a newer RAV
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 20000 --nonce 2 --sender {{$.did[0].did}}"
      Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 2"

      Then cmd: "payment-channel dispute --channel-id {{$.payment-channel[1].channel_id}} --rav {{$.payment-channel[-1].encoded}} --sender {{$.account[0].account0.address}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Fast-forward time to allow finalize cancellation after challenge period
      Then cmd: "move run --function 0x3::timestamp::fast_forward_seconds_for_local --args u64:86401"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "payment-channel finalize-cancellation --channel-id {{$.payment-channel[1].channel_id}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"
      

      Then stop the server

  @serial
  Scenario: payment_channel_receiver_close_operations
    Given a server for payment_channel_receiver_close_operations

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

    # Test 3: Create multiple RAVs for off-chain payments
    Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 15000 --nonce 1 --sender {{$.did[0].did}}"
    Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 1"

    Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 25000 --nonce 2 --sender {{$.did[0].did}}"
    Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 2"

    # Test 4: Receiver directly closes channel with RAVs (cooperative close)
    Then cmd: "payment-channel close --channel-id {{$.payment-channel[1].channel_id}} --ravs {{$.payment-channel[-2].encoded}} --ravs {{$.payment-channel[-1].encoded}} --sender {{$.account[0].account0.address}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"
    Then assert: "{{$.payment-channel[-1].ravs_count}} == 2"

    # Test 5: Query hub information for receiver after close
    Then cmd: "payment-channel query hub --owner {{$.account[0].account0.address}}"
    Then assert: "{{$.payment-channel[-1].balances[0].amount}} == 25000"

    Then stop the server
