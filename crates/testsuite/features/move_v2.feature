Feature: Rooch CLI move v2 testing

    @serial
    Scenario: move_v2_testing
        Given a server for move_v2_testing
        Then cmd: "account list --json"

        Then cmd: "move publish -p ../../examples/move_v2  --named-addresses vector_object=default --json"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

        Then cmd: "move run --function default::move_v2::call_enum"
        Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
