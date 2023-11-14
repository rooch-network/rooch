# Rooch integration test

Rooch integration test powered by Cucumber, a specification for running tests in a BDD (behavior-driven development) style workflow.

The feature current supported:

- CLI command test
- Template that supports JSON Path of previous command outputs

## How to run test

Run all the tests:

```bash
cargo test -p testsuite --test integration
```

Run a specific Scenario:

```bash
cargo test -p testsuite --test integration -- --name event
```

## How to add new test cases

Checkout `features/cmd.feature` for example.

1. Add a new CLI command test case

Use key word `Then cmd:` to add a new CLI command test case. The root command name `rooch` is not required.

For example, 

```gherkin
Then cmd: "move view --function 0x123::counter::value"
```

will run `rooch move view --function 0x123::counter::value` command and check if it will return successfully.

2. Check the output of previous command

All the previous command outputs are stored in an array named `{{$.<subcommand_name>}}`. For example, the output of command above is stored as `{{$.move[-1]}}`. You can use JSON Path to get the value of the previous command output.

The command output is like this: 
```
{
  "vm_status": "Executed",
  "return_values": [
    {
      "value": {
        "type_tag": "u64",
        "value": "0x0000000000000000"
      },
      "decoded_value": "0"
    }
  ]
}
```

You can get the value by `{{$.move[-1].return_values[0].decoded_value}}` which you can use in the next command or check if it is equal to expected value.