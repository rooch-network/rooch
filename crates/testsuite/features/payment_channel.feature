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

      # Create DID for receiver (required for revenue management)
      Then cmd: "did create self --sender {{$.account[0].account0.address}}"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

      # Test 1: Initialize payment hub with deposit
      Then cmd: "payment-channel init --owner {{$.did[0].did}} --amount 1000000000"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"


      # Test 2: Open payment channel with sub-channels (use second account as receiver)
      Then cmd: "payment-channel open --sender {{$.did[0].did}} --receiver {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Test 4: Query channel information  
      Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[1].channel_id}} --list-sub-channels"
      Then assert: "{{$.payment-channel[-1].sub_channels_count}} == 1"

      # Test 3: Create RAV (Receipt and Voucher) for off-chain payment
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 10000 --nonce 1 --sender {{$.did[0].did}}"
      Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 1"

      Then cmd: "payment-channel claim --rav {{$.payment-channel[-1].encoded}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Test 5: Query hub information for receiver (should show revenue)
      # Note: Receiver doesn't have PaymentHub, revenue goes to PaymentRevenueHub

      # Test 5.1: Query revenue balance specifically
      Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].revenue_balances[0].amount}} == 10000"
      Then assert: "{{$.payment-channel[-1].revenue_balances[0].source_type}} == payment_channel"

      # Test 5.2: Query revenue by source
      Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}} --source-type payment_channel"
      Then assert: "{{$.payment-channel[-1].total_amount}} == 10000"

      # Test 6: Withdraw revenue
      Then cmd: "payment-channel withdraw-revenue --owner {{$.did[1].did_address}} --amount 5000 --sender {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Test 6.1: Verify revenue balance after withdrawal
      Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].revenue_balances[0].amount}} == 5000"

      # Test 6.2: Verify account balance increased (withdrawn amount should be in account store)
      Then cmd: "account balance --address {{$.did[1].did_address}} --json"
      Then assert: "{{$.account[-1].RGAS.balance}} >= 5000"


      # Test 7: Cancel the channel to test cancellation workflow
      Then cmd: "payment-channel cancel --channel-id {{$.payment-channel[1].channel_id}} --sender {{$.did[0].did}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Test 8: Dispute (challenge) the cancellation with a newer RAV
      Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 20000 --nonce 2 --sender {{$.did[0].did}}"
      Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 2"

      Then cmd: "payment-channel dispute --channel-id {{$.payment-channel[1].channel_id}} --rav {{$.payment-channel[-1].encoded}} --sender {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Fast-forward time to allow finalize cancellation after challenge period
      Then cmd: "move run --function 0x3::timestamp::fast_forward_seconds_for_local --args u64:86401"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "payment-channel finalize-cancellation --channel-id {{$.payment-channel[1].channel_id}} --sender {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"
      
      # Test 9: Query revenue after dispute resolution and finalization
      Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].revenue_balances[0].amount}} == 15000"  # 5000 remaining + 10000 from dispute

      # Test 9.1: Test revenue withdrawal after dispute resolution
      Then cmd: "payment-channel withdraw-revenue --owner {{$.did[1].did_address}} --amount 10000 --sender {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

      # Test 9.2: Verify final revenue balance
      Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
      Then assert: "{{$.payment-channel[-1].revenue_balances[0].amount}} == 5000"

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

    # Create DID for receiver (required for revenue management)
    Then cmd: "did create self --sender {{$.account[0].account0.address}}"
    Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

    # Test 1: Initialize payment hub with deposit
    Then cmd: "payment-channel init --owner {{$.did[0].did}} --amount 1000000000"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    # Test 2: Open payment channel with sub-channels (use second account as receiver)
    Then cmd: "payment-channel open --sender {{$.did[0].did}} --receiver {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    # Test 3: Create multiple RAVs for off-chain payments
    Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 15000 --nonce 1 --sender {{$.did[0].did}}"
    Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 1"

    Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 25000 --nonce 2 --sender {{$.did[0].did}}"
    Then assert: "{{$.payment-channel[-1].signed_rav.sub_rav.nonce}} == 2"

    # Test 4: Receiver directly closes channel with RAVs (cooperative close)
    Then cmd: "payment-channel close --channel-id {{$.payment-channel[1].channel_id}} --ravs {{$.payment-channel[-2].encoded}} --ravs {{$.payment-channel[-1].encoded}} --sender {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"
    Then assert: "{{$.payment-channel[-1].ravs_count}} == 2"

    # Test 5: Query hub information for receiver after close
    # Note: Receiver doesn't have PaymentHub, revenue goes to PaymentRevenueHub

    # Test 5.1: Query revenue balance after channel close
    Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].revenue_balances[0].amount}} == 25000"
    Then assert: "{{$.payment-channel[-1].revenue_balances[0].source_type}} == payment_channel"

    # Test 5.2: Test full revenue withdrawal
    Then cmd: "payment-channel withdraw-revenue --owner {{$.did[1].did_address}} --amount 25000 --sender {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    # Test 5.3: Verify revenue balance is zero after full withdrawal
    Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].revenue_balances}} == []"

    # Test 5.4: Verify account balance increased by withdrawn amount
    Then cmd: "account balance --address {{$.did[1].did_address}} --json"
    Then assert: "{{$.account[-1].RGAS.balance}} >= 25000"

    Then stop the server

  @serial
  Scenario: payment_channel_revenue_management
    Given a server for payment_channel_revenue_management

    # Create test accounts and get gas first
    Then cmd: "account create"
    Then cmd: "account list --json"

    # Get gas for testing - fund the default account (sender)
    Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000000 --json --sender {{$.account[0].default.address}}"
    Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

    # Create DID for sender (required for payment channels) 
    Then cmd: "did create self"
    Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

    # Create DID for receiver (required for revenue management)
    Then cmd: "did create self --sender {{$.account[0].account0.address}}"
    Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

    # Test 1: Initialize payment hub with deposit
    Then cmd: "payment-channel init --owner {{$.did[0].did}} --amount 1000000000"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    # Test 2: Open payment channel
    Then cmd: "payment-channel open --sender {{$.did[0].did}} --receiver {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    # Test 3: Verify receiver has no revenue initially
    Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].revenue_balances}} == []"

    # Test 4: Create and claim multiple RAVs to generate revenue
    Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 10000 --nonce 1 --sender {{$.did[0].did}}"
    Then cmd: "payment-channel claim --rav {{$.payment-channel[-1].encoded}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 25000 --nonce 2 --sender {{$.did[0].did}}"
    Then cmd: "payment-channel claim --rav {{$.payment-channel[-1].encoded}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    Then cmd: "payment-channel create-rav --channel-id {{$.payment-channel[1].channel_id}} --vm-id-fragment account-key --amount 40000 --nonce 3 --sender {{$.did[0].did}}"
    Then cmd: "payment-channel claim --rav {{$.payment-channel[-1].encoded}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    # Test 5: Verify accumulated revenue
    Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].revenue_balances[0].amount}} == 40000"
    Then assert: "{{$.payment-channel[-1].revenue_balances[0].source_type}} == payment_channel"

    # Test 6: Query revenue by specific source type
    Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}} --source-type payment_channel"
    Then assert: "{{$.payment-channel[-1].total_amount}} == 40000"

    # Test 7: Partial revenue withdrawal
    Then cmd: "payment-channel withdraw-revenue --owner {{$.did[1].did_address}} --amount 15000 --sender {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    # Test 8: Verify remaining revenue balance
    Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].revenue_balances[0].amount}} == 25000"

    # Test 9: Verify account balance increased
    Then cmd: "account balance --address {{$.did[1].did_address}} --json"
    Then assert: "{{$.account[-1].RGAS.balance}} >= 15000"

    # Test 10: Try to withdraw more than available (should fail with abort_code 1)
    # Note: This command will fail with insufficient balance error, which is expected behavior
    # The test framework doesn't handle failed commands well, so we skip the assertion
    # Then cmd: "payment-channel withdraw-revenue --owner {{$.did[1].did_address}} --amount 30000 --sender {{$.did[1].did_address}}"

    # Test 11: Withdraw remaining revenue
    Then cmd: "payment-channel withdraw-revenue --owner {{$.did[1].did_address}} --amount 25000 --sender {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].execution_info.status.type}} == executed"

    # Test 12: Verify revenue balance is now zero
    Then cmd: "payment-channel query-revenue --owner {{$.did[1].did_address}}"
    Then assert: "{{$.payment-channel[-1].revenue_balances}} == []"

    # Test 13: Verify total account balance
    Then cmd: "account balance --address {{$.did[1].did_address}} --json"
    Then assert: "{{$.account[-1].RGAS.balance}} >= 40000"

    Then stop the server
