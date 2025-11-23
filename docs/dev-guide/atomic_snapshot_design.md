# Atomic Snapshot Mechanism for Rooch Pruner

## Overview

This document describes the atomic snapshot mechanism designed to solve time window race conditions in the Rooch Pruner system. The mechanism ensures that all three phases of the pruning process (BuildReach, SweepExpired, IncrementalSweep) operate on a consistent view of the chain state.

## Problem Statement

### Current Issues

1. **Time Window Race Conditions**: The current implementation creates snapshots at different times for each phase, leading to inconsistent state views.
2. **No Atomicity Guarantee**: Snapshot creation reads multiple data points separately, allowing state changes between reads.
3. **Missing Validation**: No mechanism validates that phases are using identical chain state views.
4. **No Recovery Mechanism**: Limited error handling and recovery when consistency issues arise.

### Impact

- Potential data corruption
- Incorrect pruning decisions
- System instability under load
- Difficult to debug race conditions

## Solution: Atomic Snapshot Mechanism

### Core Components

1. **AtomicSnapshotManager**: Central manager for creating, validating, and coordinating atomic snapshots
2. **ErrorRecoveryManager**: Handles error recovery and system health monitoring
3. **ValidationTests**: Comprehensive testing and validation utilities
4. **Enhanced Phase Coordination**: Modified phase implementations to use atomic snapshots

### Key Features

- **Atomic Snapshot Creation**: All snapshot data collected atomically
- **Phase Locking**: Exclusive access to snapshots during phase execution
- **Consistency Validation**: Automatic validation of snapshot integrity
- **Error Recovery**: Automatic recovery from snapshot-related errors
- **Performance Monitoring**: Comprehensive metrics and monitoring

## Architecture

### Data Flow

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   BuildReach    │    │   SweepExpired   │    │ IncrementalSweep│
│                 │    │                  │    │                 │
│ 1. Create       │    │ 1. Lock          │    │ 1. Lock         │
│    Atomic       │    │    Snapshot      │    │    Snapshot     │
│    Snapshot     │    │ 2. Validate      │    │ 2. Validate     │
│                 │    │ 3. Process       │    │ 3. Process      │
│ 2. Build        │    │ 4. Release       │    │ 4. Release      │
│    Reachability │    │    Lock          │    │    Lock         │
│                 │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                    ┌──────────────────┐
                    │ AtomicSnapshot   │
                    │ Manager          │
                    │                  │
                    │ • Create         │
                    │ • Lock           │
                    │ • Validate       │
                    │ • Release        │
                    └──────────────────┘
```

### Data Structures

#### AtomicSnapshot

```rust
pub struct AtomicSnapshot {
    pub snapshot: PruneSnapshot,
    pub snapshot_id: u64,
    pub created_at: u64,
    pub created_phase: PrunePhase,
    pub chain_metadata: ChainMetadata,
    pub integrity_hash: H256,
}
```

#### ChainMetadata

```rust
pub struct ChainMetadata {
    pub block_height: u64,
    pub chain_timestamp: u64,
    pub latest_block_hash: H256,
    pub mempool_size: Option<u64>,
    pub sync_status: Option<bool>,
}
```

#### SnapshotLock

```rust
pub struct SnapshotLock {
    pub snapshot_id: u64,
    pub locked_by_phase: PrunePhase,
    pub locked_at: u64,
    pub timeout_ms: u64,
    pub is_valid: bool,
}
```

## Implementation Details

### Atomic Snapshot Creation

1. **Atomic Data Collection**: All chain state data collected within a minimal time window
2. **Integrity Verification**: Snapshot integrity hash calculated and validated
3. **Metadata Enrichment**: Chain metadata added for comprehensive validation
4. **Persistence**: Snapshots persisted to storage for crash recovery

### Phase Locking Mechanism

1. **Exclusive Access**: Only one phase can lock a snapshot at a time
2. **Timeout Protection**: Locks have configurable timeouts to prevent deadlocks
3. **Lock Validation**: Lock integrity validated before phase execution
4. **Automatic Cleanup**: Locks automatically released on timeout or error

### Consistency Validation

1. **Internal Consistency**: Verify snapshot data coherence
2. **Cross-Phase Consistency**: Ensure all phases use same snapshot
3. **Chain State Validation**: Validate against current chain state
4. **Age Validation**: Ensure snapshots haven't expired

### Error Recovery

1. **Automatic Retry**: Configurable retry logic with exponential backoff
2. **Snapshot Recovery**: Automatic snapshot recreation on corruption
3. **Phase Rollback**: Safe rollback to previous phase on critical errors
4. **Health Monitoring**: Continuous system health monitoring

## Usage

### Basic Usage

```rust
// Create atomic snapshot manager
let snapshot_manager = Arc::new(AtomicSnapshotManager::new(
    moveos_store.clone(),
    rooch_store.clone(),
    metrics.clone(),
    Some(SnapshotManagerConfig::default()),
));

// Initialize manager
snapshot_manager.initialize()?;

// Create snapshot for BuildReach phase
let snapshot = snapshot_manager.create_snapshot(PrunePhase::BuildReach)?;

// Lock snapshot for SweepExpired phase
let locked_snapshot = snapshot_manager.lock_snapshot(PrunePhase::SweepExpired)?;

// Execute phase with locked snapshot
// ... phase logic ...

// Release lock
snapshot_manager.release_snapshot(PrunePhase::SweepExpired)?;
```

### Error Recovery Integration

```rust
// Create error recovery manager
let recovery_manager = Arc::new(ErrorRecoveryManager::new(
    snapshot_manager.clone(),
    moveos_store.clone(),
    metrics.clone(),
    Some(RecoveryConfig::default()),
));

// Execute phase with automatic recovery
let result = recovery_manager.execute_phase_with_recovery(
    PrunePhase::SweepExpired,
    || async {
        // Phase operation
        execute_sweep_expired().await
    }
).await?;
```

### Validation and Testing

```rust
// Run comprehensive validation
let validator = SnapshotValidator::new(snapshot_manager.clone(), Some(recovery_manager.clone()));
let report = validator.run_comprehensive_validation().await;

println!("Validation: {}", report.summary());
println!("{}", report.detailed_report());

// Quick health check
let health_result = quick_health_check(snapshot_manager.clone()).await;
match health_result {
    ValidationResult::Passed(_, _) => println!("System healthy"),
    ValidationResult::Failed(_, msg) => println!("System unhealthy: {}", msg),
    ValidationResult::Skipped(_, msg) => println!("Health check skipped: {}", msg),
}
```

## Configuration

### SnapshotManagerConfig

```rust
pub struct SnapshotManagerConfig {
    pub lock_timeout_ms: u64,              // 30 minutes
    pub max_snapshot_age_ms: u64,          // 2 hours
    pub enable_validation: bool,           // true
    pub enable_persistence: bool,          // true
}
```

### RecoveryConfig

```rust
pub struct RecoveryConfig {
    pub max_phase_retries: u32,            // 3
    pub base_retry_delay_ms: u64,          // 1 second
    pub max_retry_delay_ms: u64,           // 30 seconds
    pub backoff_multiplier: f64,           // 2.0
    pub enable_snapshot_recovery: bool,    // true
    pub snapshot_recovery_timeout_secs: u64, // 5 minutes
    pub enable_phase_rollback: bool,       // true
    pub min_healthy_cycles: u32,           // 3
}
```

## Performance Considerations

### Optimization Strategies

1. **Efficient Data Collection**: Minimize time between data point reads
2. **Parallel Processing**: Use parallel processing for intensive operations
3. **Caching**: Cache frequently accessed metadata
4. **Batch Operations**: Batch database operations for efficiency
5. **Memory Management**: Efficient memory usage with cleanup

### Performance Metrics

- **Snapshot Creation Time**: Target < 100ms per snapshot
- **Lock Acquisition Time**: Target < 10ms per lock
- **Validation Overhead**: Target < 5% of total execution time
- **Recovery Time**: Target < 5 minutes for full recovery

### Resource Usage

- **Memory**: ~1MB per snapshot in memory
- **Storage**: ~500KB per persisted snapshot
- **CPU**: Minimal overhead (< 2% additional CPU usage)
- **Network**: No additional network requirements

## Monitoring and Observability

### Metrics

1. **Snapshot Creation**: Creation time, success rate, failure reasons
2. **Lock Operations**: Lock acquisition time, contention rate, timeouts
3. **Validation**: Validation results, failure types, consistency issues
4. **Recovery**: Recovery attempts, success rate, recovery time
5. **System Health**: Overall health status, error rates, resource usage

### Logging

- **Debug Level**: Detailed operation traces
- **Info Level**: Phase transitions, important events
- **Warn Level**: Non-critical issues, fallback mechanisms
- **Error Level**: Critical errors, recovery failures

### Alerting

- **Snapshot Creation Failures**: Alert on > 3 consecutive failures
- **Lock Timeouts**: Alert on frequent lock timeouts
- **Consistency Validation Failures**: Alert on any validation failure
- **Recovery Failures**: Alert on recovery exhaustion

## Testing

### Unit Tests

- Snapshot creation and validation
- Lock acquisition and release
- Consistency validation logic
- Error handling and recovery

### Integration Tests

- End-to-end phase coordination
- Concurrency and race conditions
- Error scenarios and recovery
- Performance under load

### Validation Tests

- Comprehensive validation suite
- Health check functionality
- Stress testing
- Resource management testing

## Migration Guide

### From Current Implementation

1. **Update Dependencies**: Add new dependencies to Cargo.toml
2. **Replace Phase Logic**: Update phase implementations to use atomic snapshots
3. **Add Initialization**: Initialize AtomicSnapshotManager
4. **Add Monitoring**: Update metrics collection
5. **Update Configuration**: Add new configuration options

### Backward Compatibility

- **Fallback Mechanism**: Fallback to basic snapshot creation if atomic snapshot fails
- **Gradual Migration**: Can be rolled out incrementally
- **Feature Flags**: Can be disabled via configuration

## Future Enhancements

### Planned Features

1. **Distributed Snapshots**: Support for distributed snapshot coordination
2. **Advanced Caching**: Multi-level caching for better performance
3. **Machine Learning**: Predictive error prevention
4. **Enhanced Monitoring**: Real-time dashboard integration
5. **API Integration**: REST API for external monitoring

### Performance Improvements

1. **Lock-Free Algorithms**: Implement lock-free data structures where possible
2. **Memory Pooling**: Reduce memory allocation overhead
3. **Compression**: Compress persisted snapshots to reduce storage
4. **Incremental Updates**: Support for incremental snapshot updates

## Conclusion

The atomic snapshot mechanism provides a robust solution to race conditions in the Rooch Pruner system. By ensuring consistent state views across all phases and providing comprehensive error handling and recovery, it significantly improves system reliability and maintainability.

The implementation is designed to be production-ready with comprehensive monitoring, testing, and documentation. It provides both immediate benefits and a foundation for future enhancements.