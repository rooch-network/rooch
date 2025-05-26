Feature: Rooch CLI DID integration tests

    @serial
    Scenario: did_operations
      Given a server for did_operations

      # Create test accounts
      Then cmd: "account create"
      Then cmd: "account list --json"

      # Test 1: Create DID for self
      Then cmd: "did create self"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"

      # Test 2: Query DID by address
      Then cmd: "did query address {{$.did[0].did_address}}"
      Then assert: "{{$.did[-1].did_document.id.method}} == rooch"

      # Test 3: Query DID by DID string
      Then cmd: "did query did {{$.did[0].did}}"
      Then assert: "{{$.did[-1].object_id}} == {{$.did[-3].object_id}}"

      # Test 4: Check if DID exists
      Then cmd: "did query exists {{$.did[0].did}}"
      Then assert: "{{$.did[-1].exists}} == true"
      Then assert: "{{$.did[-1].query_type}} == did_identifier"

      # Test 5: Query DID by ObjectID
      Then cmd: "did query object-id {{$.did[0].object_id}}"
      Then assert: "{{$.did[-1].object_id}} == {{$.did[0].object_id}}"



      # Test 6: Add verification method to DID
      Then cmd: "did manage add-vm --did-address {{$.did[0].did_address}} --fragment key-2 --method-type Ed25519VerificationKey2020 --relationships auth,assert"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"
      Then assert: "{{$.did[-1].operation}} == add_verification_method"
      Then assert: "{{$.did[-1].fragment}} == key-2"

      # Test 7: Add verification method to relationship
      Then cmd: "did manage add-relationship --did-address {{$.did[0].did_address}} --fragment key-2 --relationship invoke"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"
      Then assert: "{{$.did[-1].operation}} == add_to_invoke"


      # Test 8: Remove verification method from relationship
      Then cmd: "did manage remove-relationship --did-address {{$.did[0].did_address}} --fragment key-2 --relationship assert"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"
      Then assert: "{{$.did[-1].operation}} == remove_from_assert"

      # Test 9: Remove verification method
      Then cmd: "did manage remove-vm --did-address {{$.did[0].did_address}} --fragment key-2"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"
      Then assert: "{{$.did[-1].operation}} == remove_verification_method"



      # Test 10: Add service to DID
      Then cmd: "did manage add-service --did-address {{$.did[0].did_address}} --fragment test_service --service-type MessagingService --endpoint https://example.com/messaging --properties priority=high,encryption=true"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"
      Then assert: "{{$.did[-1].operation}} == add_service"
      Then assert: "{{$.did[-1].fragment}} == test_service"


      # Test 11: Update service
      Then cmd: "did manage update-service --did-address {{$.did[0].did_address}} --fragment test_service --service-type CadopCustodianService --endpoint https://new.example.com/messaging --properties priority=medium"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"
      Then assert: "{{$.did[-1].operation}} == update_service"


      # Test 12: generate keys
      Then cmd: "did keygen ed25519"
      Then cmd: "did keygen secp256k1"
      

      # Test 13: cadop - use generated keys
      Then cmd: "did create cadop --user-did-key {{$.did[-2].did_key}} --custodian-service-key {{$.did[-1].public_key.raw_multibase}} --custodian-key-type EcdsaSecp256k1VerificationKey2019 --sender {{$.did[0].did_address}}"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"


      # Test 14: Remove service
      Then cmd: "did manage remove-service --did-address {{$.did[0].did_address}} --fragment test_service"
      Then assert: "{{$.did[-1].execution_info.status.type}} == executed"
      Then assert: "{{$.did[-1].operation}} == remove_service"

      # Test 15: Query DID documents controlled by controller
      Then cmd: "did query controller {{$.did[-4].did_key}}"
      Then assert: "{{$.did[-1].controller}} == {{$.did[-5].did_key}}"

      Then stop the server
