use crate::statedb_pruner::StatedbPruner;
use std::sync::Arc;
use moveos_types::h256::H256;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn start_server(opt: RoochOpt, server_opt: ServerOpt) -> Result<ServerHandle> {
    // ... existing server setup code ...

    // Initialize statedb pruner with the node store
    let node_store = Arc::new(opt.state_store.node_store.clone());
    let pruner = Arc::new(StatedbPruner::new(node_store));
    
    // Register current state root
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    pruner.register_state_root(opt.state_store.current_state_root(), current_timestamp);
    
    // Start pruning task in background
    let pruner_clone = pruner.clone();
    tokio::spawn(async move {
        pruner_clone.start_pruning().await;
    });

    // Store pruner in server handle for later use
    let server_handle = ServerHandle {
        shutdown_tx,
        timers,
        _opt: opt,
        _prometheus_registry: prometheus_registry,
        pruner,
    };

    // ... rest of existing server code ...

    Ok(server_handle)
}

// Update ServerHandle to include pruner
pub struct ServerHandle {
    shutdown_tx: Sender<()>,
    timers: Vec<Timer>,
    _opt: RoochOpt,
    _prometheus_registry: prometheus::Registry,
    pruner: Arc<StatedbPruner>,
}

impl ServerHandle {
    // Add method to update latest state
    pub fn update_latest_state(&self, field_key: FieldKey, object_state: &ObjectState) {
        self.pruner.update_latest_state(field_key, object_state);
    }

    // Add method to register new state root
    pub fn register_state_root(&self, state_root: H256) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        self.pruner.register_state_root(state_root, timestamp);
        Ok(())
    }
} 