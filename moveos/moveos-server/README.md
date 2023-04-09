# MoveOS Server

MoveOS Server is a standalone Move runtime environment based on [MoveOS](./moveos).

## Usage

1. Build the crate, run `cargo build -p moveos-server`.

2. Start the MoveOS Server, run `moveos-server start` with default config, or `ROOCH_CONFIG=./rooch.yml ./target/debug/moveos-server start` if you can specific the config path or use the default value, can print the text like `Listening on 0.0.0.0:50051`.
   
3. Test the server, run `./target/debug/moveos-server say --name Rooch`, if the server is works, the output text should look like:
   ```
    HelloResponse { 
        message: "response Rooch, a2a83ef4-fb09-488a-8977-188b5b9ed2cf", 
        timestamp: Some(Timestamp { seconds: 1681019952, nanos: 231311895 }) 
    }
    ```