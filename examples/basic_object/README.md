# A Basic Object Example

## Start local server

1. add fixtures/config.yml to the ROOCH_CONFIG environment variable.
2. Start a local server:
```shell
rooch server start
```

## Publish the example

Open another terminal, publish this example (Note that the placeholder `{ACCOUNT_ADDRESS}` should be replaced with the address of your account):

```shell
# rooch account list # List your accounts, pick one
# rooch account create # Create a account if no accounts listed
rooch move publish --named-addresses rooch_examples={ACCOUNT_ADDRESS}
```

## Run functions

Run a function to create something on-chain: 

```shell
rooch move run --function {ACCOUNT_ADDRESS}::something_aggregate::create_something --args 1u32 2u128 --sender-account {ACCOUNT_ADDRESS}
```

## Query using JSON RPC APIs

### Get events by event handle type

Run this command to get events by event handle type:

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params": [
    "{EVENT_HANDLE_TYPE}", "{CURSOR}", "{LIMIT}"
]
}'
```

An example:

```
curl --location 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data '{
    "id": 101,
    "jsonrpc": "2.0",
    "method": "rooch_getEventsByEventHandle",
    "params": [
        "0xb4321fa441b5d9fdefb71f82856a56447451f7b1ba9478747b07e9f26b34c87::something::SomethingCreated", 1, 2
    ]
}'
```

The output is similar to the following (Note that the ID of the created object appears where the placeholder `{ID_OF_CREATED_OBJECT}` is located.):

```json
{"jsonrpc":"2.0","result":[{"sender":"0000000000000000000000000000000000000000000000000000000000000000","event_data":"0x0b00395f380aa20ab634291b1fe8705e8ba94ce5bfab66dbe436865cc40974ef0100000002000000000000000000000000000000","parsed_event_data":{"abilities":8,"type":"0x565d5717526aecec1f9d464867f7d92d6eae2dc8ca73a0dc2613dd185d3d7bc7::something::SomethingCreated","value":{"i":1,"j":"2","obj_id":"{ID_OF_CREATED_OBJECT}"}},"type_tag":"0x565d5717526aecec1f9d464867f7d92d6eae2dc8ca73a0dc2613dd185d3d7bc7::something::SomethingCreated","event_index":3,"event_id":{"event_handle_id":"0x53f32af12dc9236eb67f1c064cf55ee8891a90040f71ba17422cfdd91eb7358b","event_seq":0}}],"id":101}
```

### Get annotated states by object ID

To retrieve information of an object through the RPC interface (The placeholder `OBJECT_ID` should be replaced with the value output above.):

```shell
curl --location --request POST 'http://127.0.0.1:50051/' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getAnnotatedStates",
 "params":["/object/{OBJECT_ID}"]
}'
```

The output is similar to the following (Note that the handle of the Table embedded in the created object appears where the placeholder `{HANDLE_OF_CREATED_OBJECT_TABLE}` is located.):

```json
{"jsonrpc":"2.0","result":[{"state":{"value":"0x0b00395f380aa20ab634291b1fe8705e8ba94ce5bfab66dbe436865cc40974ef565d5717526aecec1f9d464867f7d92d6eae2dc8ca73a0dc2613dd185d3d7bc70100000002000000000000000000000000000000ba90d115eab89e3167e4fb9a489a46606189e8ad474d5e232fd70568923effff0b64dc6ef8063f3819a2458643c86d2869dfc5064b6e33212ca27742887d6dc0","value_type":"0x1::object::Object<0x565d5717526aecec1f9d464867f7d92d6eae2dc8ca73a0dc2613dd185d3d7bc7::something::SomethingProperties>"},"move_value":{"abilities":0,"type":"0x1::object::Object<0x565d5717526aecec1f9d464867f7d92d6eae2dc8ca73a0dc2613dd185d3d7bc7::something::SomethingProperties>","value":{"id":"0xb00395f380aa20ab634291b1fe8705e8ba94ce5bfab66dbe436865cc40974ef","owner":"0x565d5717526aecec1f9d464867f7d92d6eae2dc8ca73a0dc2613dd185d3d7bc7","value":{"abilities":8,"type":"0x565d5717526aecec1f9d464867f7d92d6eae2dc8ca73a0dc2613dd185d3d7bc7::something::SomethingProperties","value":{"barTable":{"abilities":4,"type":"0x1::table::Table<u8, u128>","value":{"handle":"{HANDLE_OF_CREATED_OBJECT_TABLE}"}},"fooTable":{"abilities":4,"type":"0x1::table::Table<0x1::string::String, 0x1::string::String>","value":{"handle":"0xba90d115eab89e3167e4fb9a489a46606189e8ad474d5e232fd70568923effff"}},"i":1,"j":"2"}}}}}],"id":101}
```

### Get table item

To retrieve value of a table item through the RPC interface `rooch_getAnnotatedStates` (The placeholder `TABLE_HANDLE` should be replaced with the value output above.):

```shell
curl --location --request POST 'http://127.0.0.1:50051/' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getAnnotatedStates",
 "params":["/table/{TABLE_HANDLE}/0x01"]
}'
```

The output is similar to the following:

```json
{"jsonrpc":"2.0","result":[{"state":{"value":"0x01000000000000000000000000000000","value_type":"u128"},"move_value":"1"}],"id":101}
```
