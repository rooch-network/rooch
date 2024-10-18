Feature: Rooch CLI vector object integration tests

    @serial
    Scenario: vector_object_test
        Given a server for vector_object_test
        Then cmd: "account list --json"

        Then cmd: "move publish -p ../../examples/vector_object  --named-addresses vector_object=default --json"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move run --function default::vector_object::create_mock_object_to_sender"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
        Then sleep: "3"
        Then cmd: "object -o default -t {{$.account[-1].default.hex_address}}::vector_object::MockObject"
        Then cmd: "move run --function default::vector_object::transfer_vector_object --args vector<object>:{{$.object[-1].data[0].id}}"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move run --function default::vector_object::create_mock_object_to_user --sender=default --args address:0xbb3d042321a986b65bdd03db16d32af0d3474ba4f295820e324c4af50bfa9962"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
        Then sleep: "3"
        Then cmd: "object -o 0xbb3d042321a986b65bdd03db16d32af0d3474ba4f295820e324c4af50bfa9962 -t {{$.account[-1].default.hex_address}}::vector_object::MockObject"
        Then cmd: "move run --function default::vector_object::transfer_vector_object --args vector<object>:{{$.object[-1].data[0].id}}"
        Then assert: "'{{$.move[-1]}}' contains NO_ACCOUNT_ROLE"

    @serial
    Scenario: vector_named_object_test_1
        Given a server for vector_object_test
        Then cmd: "account list --json"

        Then cmd: "move publish -p ../../examples/vector_object  --named-addresses vector_object=default --json"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move run --function default::vector_object::create_named_mock_object_to_sender"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
        Then sleep: "3"
        Then cmd: "object -o default -t {{$.account[-1].default.hex_address}}::vector_object::MockObject"
        Then cmd: "move run --function default::vector_object::transfer_vector_object --args vector<object>:default::vector_object::MockObject"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

    @serial
        Scenario: vector_named_object_test_2
        Given a server for vector_object_test
        Then cmd: "account list --json"

        Then cmd: "move publish -p ../../examples/vector_object  --named-addresses vector_object=default --json"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move run --function default::vector_object::create_named_mock_object_to_user --args address:0xbb3d042321a986b65bdd03db16d32af0d3474ba4f295820e324c4af50bfa9962"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
        Then sleep: "3"
        Then cmd: "object -o 0xbb3d042321a986b65bdd03db16d32af0d3474ba4f295820e324c4af50bfa9962 -t {{$.account[-1].default.hex_address}}::vector_object::MockObject"
        Then cmd: "move run --function default::vector_object::transfer_vector_object --args vector<object>:default::vector_object::MockObject"
        Then assert: "'{{$.move[-1]}}' contains NO_ACCOUNT_ROLE"