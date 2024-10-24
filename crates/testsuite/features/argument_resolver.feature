Feature: Rooch CLI argument resolver integration tests

    @serial
    Scenario: argument_resolver_test
        Given a server for vector_object_test
        Then cmd: "account list --json"

        Then cmd: "move publish -p ../../examples/argument_resolver  --named-addresses argument_resolver=default --json"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move run --function default::argument_resolver::create_mock_object_to_sender --args u64:123"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
        Then sleep: "3"
        Then cmd: "object -o default -t {{$.account[-1].default.hex_address}}::argument_resolver::MockObject"
        Then cmd: "move run --function default::argument_resolver::object --args object:{{$.object[-1].data[0].id}} --args u64:123"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move run --function default::argument_resolver::create_mock_object_to_sender --args u64:123"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
        Then sleep: "3"
        Then cmd: "object -o default -t {{$.account[-1].default.hex_address}}::argument_resolver::MockObject"
        Then cmd: "move run --function default::argument_resolver::object_ref --args object:{{$.object[-1].data[0].id}} --args u64:123"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move run --function default::argument_resolver::create_mock_object_to_sender --args u64:123"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
        Then sleep: "3"
        Then cmd: "object -o default -t {{$.account[-1].default.hex_address}}::argument_resolver::MockObject"
        Then cmd: "move run --function default::argument_resolver::object_mut_ref --args object:{{$.object[-1].data[0].id}} --args u64:123"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move view --function default::argument_resolver::vector_string_argument --args vector<string>:hello123"
        Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 'hello123'"

        Then cmd: "move view --function default::argument_resolver::vector_object_id_argument --args vector<object_id>:0x0a3e8d86b65c8d51ffa92ed278ac96f895a4b7c8bdb60a8b6cd8a5393cb27760"
        Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 0x0a3e8d86b65c8d51ffa92ed278ac96f895a4b7c8bdb60a8b6cd8a5393cb27760"

        Then cmd: "move view --function default::argument_resolver::string_argument --args string:hello123"
        Then assert: "{{$.move[-1].return_values[0].decoded_value}} == 'hello123'"

        Then cmd: "move view --function default::argument_resolver::object_id_argument --args object_id:0x0a3e8d86b65c8d51ffa92ed278ac96f895a4b7c8bdb60a8b6cd8a5393cb27760"
        Then assert: "{{$.move[-1].return_values[0].decoded_value}} == '0x0a3e8d86b65c8d51ffa92ed278ac96f895a4b7c8bdb60a8b6cd8a5393cb27760'"