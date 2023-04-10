# MoveOS server

## Description
Moveos server is a standalone service based on [MoveOS](https://github.com/rooch-network/rooch/blob/moveos_server/moveos/moveos-server/moveos), which includes move vm, statedb, moveos framework, etc.
Moveos server is also standard MoveVM runtime, you can publish the Move module to Moveos server, and call the *entry functions(execute transaction)* on Moveos server .

1. Moveos server listens to and receives requests from the client through the grpc service, 
2. and then delegates the requests to the proxy to process, the proxy including moveos(state and runtime) and actor (executor), 
3. and then the actor performs the specific tasks and returns the results to the client. 
If parallel processing is required, the actor can provide good parallelism.

Moveos server provides a default configuration file in the current directory, including ip and port, 
or you  can specify a different configuration file if you need to use a different configuration.

### TODO:
1. moveos server receives publish module and handle.
2. moveos server receives entry function from cli and execute transaction.
3. *publish module* and *entry function* protocol between cli and server.

## Build and run
1. Build the crate, run `cargo build -p moveos-server` for standalone, or `cargo build` for (rooch) 

2. Start the MoveOS server, run `moveos-server start` with default config, or `ROOCH_CONFIG=./rooch.yml ./target/debug/moveos-server start` if you can specific the config path or use the default value, can print the text like Listening on 0.0.0.0:50051.
   or run `rooch server start` with default config, or `ROOCH_CONFIG=./rooch.yml ./target/debug/rooch server start`.

3. Test the server, run `./target/debug/moveos-server say --name Rooch`, if the server is works, the output text should look like:
   ```
    HelloResponse { 
        message: "response Rooch, a2a83ef4-fb09-488a-8977-188b5b9ed2cf", 
        timestamp: Some(Timestamp { seconds: 1681019952, nanos: 231311895 }) 
    }
    ```