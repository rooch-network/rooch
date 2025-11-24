# Rooch Pruner End-to-End Testing Guide

## 1. Overview

This guide provides a comprehensive approach for end-to-end testing of the Rooch Pruner functionality. The pruner is a critical component that manages the state database size by identifying and removing unreachable nodes. This document outlines how to set up automated testing using the TypeScript SDK's E2E framework and Prometheus metrics for validation.

### 1.1 Purpose

The pruner E2E tests serve several important purposes:

- **Validate Pruner Functionality**: Ensure all three pruner phases (BuildReach, SweepExpired, IncrementalSweep) work correctly
- **Performance Monitoring**: Track pruning effectiveness through metrics collection
- **Regression Testing**: Catch issues before they affect production environments
- **Documentation**: Provide examples of pruner behavior under different workloads

### 1.2 Test Scenarios

The E2E tests cover three main scenarios:

1. **State Modification (Counter)**: Tests repeated modifications of the same state object, generating stale nodes for incremental sweep
2. **Object Lifecycle**: Tests object creation, updates, and deletion to verify full lifecycle pruning
3. **Mixed Workload**: Combines both scenarios to simulate real-world application patterns

## 2. Architecture

### 2.1 Prometheus Metrics Infrastructure

The testing framework leverages Rooch's built-in Prometheus metrics infrastructure:

- **Metrics Server**: Runs on port 9184 by default (`http://localhost:9184/metrics`)
- **Pruner Metrics**: Comprehensive metrics exported by the pruner component
- **Collection**: TypeScript client polls metrics endpoint during test execution
- **Validation**: Automated checks against expected metric values and patterns

### 2.2 Pruner Metrics Overview

The pruner exports the following key metrics:

| Metric Name | Type | Description |
|-------------|------|-------------|
| `pruner_current_phase` | Gauge | Current pruning phase (0=BuildReach, 1=SweepExpired, 2=Incremental) |
| `pruner_reachable_nodes_scanned` | Histogram | Nodes scanned during reachability analysis |
| `pruner_sweep_nodes_deleted` | Histogram | Nodes deleted during sweep operations |
| `pruner_bloom_filter_size_bytes` | Gauge | Bloom filter memory usage |
| `pruner_disk_space_reclaimed_bytes` | Counter | Total disk space reclaimed |
| `pruner_processing_speed_nodes_per_sec` | Histogram | Processing speed per phase |
| `pruner_error_count` | Counter | Number of errors encountered |

### 2.3 Test Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Test Script   │────│  Prometheus      │────│   Rooch Node    │
│   (TypeScript)  │    │   Metrics API    │    │   (with Pruner) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
    ┌────▼────┐             ┌────▼────┐             ┌────▼────┐
    │  Test   │             │ Metrics │             │  State   │
    │  Boxes  │             │ Client  │             │   DB     │
    └────┬────┘             └────┬────┘             └────┬────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │     Test Validation     │
                    │   & Report Generation   │
                    └─────────────────────────┘
```

## 3. Prerequisites

### 3.1 Environment Setup

- **Rooch CLI**: Installed and configured for local development
- **Node.js**: Version 18+ with pnpm package manager
- **Rust**: For building Rooch components
- **Docker**: For TestBox container management (optional)

### 3.2 Dependencies

The following components must be available:

```bash
# Rooch components
rooch-framework/
rooch-rpc-server/
rooch-pruner/

# TypeScript SDK
sdk/typescript/test-suite/
sdk/typescript/rooch-sdk/
```

### 3.3 Network Configuration

Tests run against a local Rooch network with pruner enabled:

```bash
# Pruner configuration example
--pruner-enable true
--pruner-interval-s 30
--pruner-window-days 0
--pruner-enable-incremental-sweep true
--pruner-bloom-bits 1073741824
```

## 4. Implementation Steps

### 4.1 Enable Pruner Metrics Export

**File**: `crates/rooch-rpc-server/src/lib.rs`

Modify the `run_start_server` function to enable pruner metrics:

```rust:crates/rooch-rpc-server/src/lib.rs
// Line ~202: After prometheus registry creation
let prometheus_registry = start_basic_prometheus_server();
// Initialize metrics before creating any stores
init_metrics(&prometheus_registry);

// Add: Create pruner metrics instance
let pruner_metrics = Arc::new(rooch_pruner::metrics::PrunerMetrics::new(&prometheus_registry));

// Line ~221: Modify pruner startup
let pruner = StatePruner::start(
    Arc::new(opt.pruner.clone()),
    Arc::new(moveos_store.clone()),
    Arc::new(rooch_store.clone()),
    shutdown_tx.subscribe(),
    Some(pruner_metrics), // Enable metrics collection
)?;
```

**Verification**:
```bash
# Check that pruner metrics are exported
curl http://localhost:9184/metrics | grep pruner_
```

### 4.2 Create Test Contracts

**File**: `examples/pruner_test/sources/object_lifecycle.move`

Create a Move contract for object lifecycle testing:

```move
module pruner_test::object_lifecycle {
    use moveos_std::object::{Self, Object};
    use moveos_std::tx_context;

    struct TestObject has key, store {
        value: u64,
        data: vector<u8>,
    }

    /// Create a new object with specified value and data size
    public entry fun create_object(value: u64, size: u64) {
        let mut data = vector::empty<u8>();
        let mut i = 0;
        while (i < size) {
            vector::push_back(&mut data, ((i % 256) as u8));
            i = i + 1;
        };

        let obj = object::new(TestObject { value, data });
        object::transfer(obj, tx_context::sender());
    }

    /// Remove an object (creates stale nodes)
    public entry fun remove_object(obj: Object<TestObject>) {
        let TestObject { value: _, data: _ } = object::remove(obj);
    }

    /// Update object value (creates new version, old becomes stale)
    public entry fun update_object(obj: &mut Object<TestObject>, new_value: u64) {
        let test_obj = object::borrow_mut(obj);
        test_obj.value = new_value;
    }
}
```

**Package Configuration**: `examples/pruner_test/Move.toml`

```toml
[package]
name = "pruner_test"
version = "0.1.0"

[dependencies]
MoveStdlib = { git = "https://github.com/move-language/move.git", subdir = "move-stdlib", rev = "main" }
MoveosStdlib = { git = "https://github.com/rooch-network/moveos.git", subdir = "moveos-stdlib", rev = "main" }
RoochFramework = { local = "../frameworks/rooch-framework" }

[addresses]
pruner_test = "_"
```

### 4.3 Implement Prometheus Client

**File**: `sdk/typescript/test-suite/src/utils/prometheus-client.ts`

Create a TypeScript client for collecting metrics:

```typescript:sdk/typescript/test-suite/src/utils/prometheus-client.ts
export interface PrunerMetrics {
  // Current phase (0=BuildReach, 1=SweepExpired, 2=Incremental)
  currentPhase: number

  // Node deletion statistics
  sweepExpiredDeleted: { sum: number; count: number }
  incrementalSweepDeleted: { sum: number; count: number }

  // Scanning statistics
  reachableNodesScanned: { sum: number; count: number }

  // Resource usage
  bloomFilterSizeBytes: number
  diskSpaceReclaimedBytes: number

  // Performance
  processingSpeedNodesPerSec: { sum: number; count: number }

  // Errors
  errorCount: number
}

export class PrometheusClient {
  constructor(private port: number = 9184) {}

  async fetchMetrics(): Promise<PrunerMetrics> {
    const response = await fetch(`http://localhost:${this.port}/metrics`)
    const text = await response.text()

    const metrics: Record<string, number> = {}
    for (const line of text.split('\n')) {
      if (line.startsWith('#') || !line.trim()) continue

      // Parse Prometheus exposition format
      const match = line.match(/^([a-z_]+(?:\{[^}]*\})?)\s+([\d.eE+-]+)/)
      if (match) {
        const [, nameWithLabels, value] = match
        metrics[nameWithLabels] = parseFloat(value)
      }
    }

    return {
      currentPhase: this.extractMetric(metrics, 'pruner_current_phase'),
      sweepExpiredDeleted: this.extractHistogram(metrics, 'pruner_sweep_nodes_deleted', 'SweepExpired'),
      incrementalSweepDeleted: this.extractHistogram(metrics, 'pruner_sweep_nodes_deleted', 'Incremental'),
      reachableNodesScanned: this.extractHistogram(metrics, 'pruner_reachable_nodes_scanned', 'BuildReach'),
      bloomFilterSizeBytes: this.extractGauge(metrics, 'pruner_bloom_filter_size_bytes'),
      diskSpaceReclaimedBytes: this.extractCounter(metrics, 'pruner_disk_space_reclaimed_bytes'),
      processingSpeedNodesPerSec: this.extractHistogram(metrics, 'pruner_processing_speed_nodes_per_sec'),
      errorCount: this.extractCounter(metrics, 'pruner_error_count'),
    }
  }

  private extractMetric(metrics: Record<string, number>, name: string): number {
    return metrics[name] || 0
  }

  private extractGauge(metrics: Record<string, number>, name: string, label?: string): number {
    const key = label ? `${name}{phase="${label}"}` : name
    return metrics[key] || 0
  }

  private extractCounter(metrics: Record<string, number>, name: string): number {
    return metrics[name] || 0
  }

  private extractHistogram(metrics: Record<string, number>, name: string, label?: string) {
    const labelStr = label ? `{phase="${label}"}` : ''
    return {
      sum: metrics[`${name}_sum${labelStr}`] || 0,
      count: metrics[`${name}_count${labelStr}`] || 0,
    }
  }
}
```

### 4.4 Implement E2E Tests

**File**: `sdk/typescript/test-suite/src/e2e/pruner-e2e.spec.ts`

Create the main E2E test suite:

```typescript:sdk/typescript/test-suite/src/e2e/pruner-e2e.spec.ts
import { TestBox } from '../testbox'
import { PrometheusClient, PrunerMetrics } from '../utils/prometheus-client'

describe('Rooch Pruner E2E Test', () => {
  let testbox: TestBox
  let prometheus: PrometheusClient
  const metricsPort = 9184

  beforeAll(async () => {
    // Start node with pruner configuration
    testbox = new TestBox()
    await testbox.loadRoochEnv('local', undefined, [
      '--pruner-enable', 'true',
      '--pruner-interval-s', '30',
      '--pruner-window-days', '0',
      '--pruner-enable-incremental-sweep', 'true',
      '--pruner-bloom-bits', '1073741824', // 1GB
    ])

    prometheus = new PrometheusClient(metricsPort)
  }, 120000) // 2 minute timeout

  afterAll(async () => {
    await testbox.stop()
  })

  describe('Scenario 1: State Modification (Counter)', () => {
    it('should generate stale nodes and trigger incremental sweep', async () => {
      const iterations = 500

      // Send counter transactions
      for (let i = 0; i < iterations; i++) {
        await testbox.getClient().executeViewFunction({
          target: '0x3::counter::increase',
        })
        if (i % 50 === 0) {
          console.log(`Counter iterations: ${i}/${iterations}`)
        }
        await delay(50)
      }

      // Wait for pruner to run
      await delay(60000) // 60 seconds

      // Validate metrics
      const metrics = await prometheus.fetchMetrics()

      expect(metrics.incrementalSweepDeleted.count).toBeGreaterThan(0)
      expect(metrics.incrementalSweepDeleted.sum).toBeGreaterThan(0)

      console.log('Incremental sweep stats:', {
        iterations: metrics.incrementalSweepDeleted.count,
        nodesDeleted: metrics.incrementalSweepDeleted.sum,
      })
    }, 120000)
  })

  describe('Scenario 2: Object Lifecycle', () => {
    it('should handle object creation and deletion', async () => {
      const iterations = 100

      // Deploy test contract
      await testbox.publishPackage('../../examples/pruner_test')

      const objectIds: string[] = []

      for (let i = 0; i < iterations; i++) {
        // Create object
        const createResult = await testbox.getClient().executeViewFunction({
          target: '0x3::pruner_test::object_lifecycle::create_object',
          args: [i, 1024], // value=i, size=1KB
        })

        // Extract object ID from transaction result
        const objectId = extractObjectIdFromResult(createResult)
        objectIds.push(objectId)

        await delay(50)

        if (i % 20 === 0) {
          console.log(`Object lifecycle iterations: ${i}/${iterations}`)
        }
      }

      // Wait for pruner processing
      await delay(90000) // 90 seconds

      // Validate pruner behavior
      const metrics = await prometheus.fetchMetrics()

      // Verify at least one complete pruner cycle
      expect(metrics.currentPhase).toBeGreaterThanOrEqual(0)
      expect(metrics.diskSpaceReclaimedBytes).toBeGreaterThan(0)

      console.log('Pruner summary:', {
        phase: metrics.currentPhase,
        reclaimedBytes: metrics.diskSpaceReclaimedBytes,
        bloomFilterBytes: metrics.bloomFilterSizeBytes,
        errors: metrics.errorCount,
      })
    }, 180000)
  })

  describe('Metrics Validation', () => {
    it('should export all expected metrics', async () => {
      const metrics = await prometheus.fetchMetrics()

      // Check all metric fields exist
      expect(metrics).toHaveProperty('currentPhase')
      expect(metrics).toHaveProperty('sweepExpiredDeleted')
      expect(metrics).toHaveProperty('incrementalSweepDeleted')
      expect(metrics).toHaveProperty('bloomFilterSizeBytes')
      expect(metrics).toHaveProperty('diskSpaceReclaimedBytes')

      // Bloom filter should be initialized
      expect(metrics.bloomFilterSizeBytes).toBeGreaterThan(0)
    })
  })
})

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function extractObjectIdFromResult(result: any): string {
  // Extract object ID from transaction result
  // Implementation depends on actual transaction result structure
  return result.objectId || 'mock-object-id'
}
```

### 4.5 Implement Test Reporter

**File**: `sdk/typescript/test-suite/src/utils/test-reporter.ts`

Create a test report generator:

```typescript:sdk/typescript/test-suite/src/utils/test-reporter.ts
import { PrunerMetrics } from './prometheus-client'

export interface TestReport {
  timestamp: string
  duration: number
  transactions: {
    counter: number
    objectCreated: number
    objectDeleted: number
  }
  prunerMetrics: PrunerMetrics
  validation: {
    passed: boolean
    checks: Record<string, boolean>
  }
}

export function generateReport(
  startTime: number,
  txCounts: { counter: number; objectCreated: number; objectDeleted: number },
  metrics: PrunerMetrics
): TestReport {
  const checks = {
    prunerStarted: metrics.currentPhase >= 0,
    incrementalSweepActive: metrics.incrementalSweepDeleted.count > 0,
    nodesDeleted: (metrics.sweepExpiredDeleted.sum + metrics.incrementalSweepDeleted.sum) > 0,
    spaceReclaimed: metrics.diskSpaceReclaimedBytes > 0,
    noErrors: metrics.errorCount === 0,
  }

  return {
    timestamp: new Date().toISOString(),
    duration: Date.now() - startTime,
    transactions: txCounts,
    prunerMetrics: metrics,
    validation: {
      passed: Object.values(checks).every(v => v),
      checks,
    },
  }
}

export function printReport(report: TestReport): void {
  console.log('\n=== Rooch Pruner E2E Test Report ===')
  console.log(`Timestamp: ${report.timestamp}`)
  console.log(`Duration: ${(report.duration / 1000).toFixed(1)}s`)
  console.log('\nTransactions:')
  console.log(`  Counter: ${report.transactions.counter}`)
  console.log(`  Objects Created: ${report.transactions.objectCreated}`)
  console.log(`  Objects Deleted: ${report.transactions.objectDeleted}`)
  console.log('\nPruner Metrics:')
  console.log(`  Current Phase: ${report.prunerMetrics.currentPhase}`)
  console.log(`  Nodes Deleted (Sweep): ${report.prunerMetrics.sweepExpiredDeleted.sum}`)
  console.log(`  Nodes Deleted (Incremental): ${report.prunerMetrics.incrementalSweepDeleted.sum}`)
  console.log(`  Disk Space Reclaimed: ${(report.prunerMetrics.diskSpaceReclaimedBytes / 1024 / 1024).toFixed(2)} MB`)
  console.log(`  Bloom Filter Size: ${(report.prunerMetrics.bloomFilterSizeBytes / 1024 / 1024).toFixed(2)} MB`)
  console.log(`  Errors: ${report.prunerMetrics.errorCount}`)
  console.log('\nValidation:')
  for (const [check, passed] of Object.entries(report.validation.checks)) {
    console.log(`  ${passed ? '✅' : '❌'} ${check}`)
  }
  console.log(`\nOverall: ${report.validation.passed ? '✅ PASSED' : '❌ FAILED'}`)
}
```

## 5. Running Tests

### 5.1 Prerequisites Setup

```bash
# Build Rooch with pruner metrics enabled
cd /path/to/rooch
make build

# Install TypeScript dependencies
cd sdk/typescript/test-suite
pnpm install
```

### 5.2 Run Complete Test Suite

```bash
# Run all pruner E2E tests
pnpm test -- pruner-e2e

# Run with verbose output
pnpm test -- pruner-e2e --verbose

# Run specific test scenario
pnpm test -- pruner-e2e -t "State Modification"
pnpm test -- pruner-e2e -t "Object Lifecycle"
```

### 5.3 Run Individual Tests

```bash
# Run only counter scenario
pnpm test -- pruner-e2e.spec.ts -t "should generate stale nodes"

# Run only object lifecycle scenario
pnpm test -- pruner-e2e.spec.ts -t "should handle object creation"
```

### 5.4 Manual Metrics Verification

```bash
# Check metrics endpoint directly
curl http://localhost:9184/metrics | grep pruner

# Monitor metrics during test execution
watch -n 5 'curl -s http://localhost:9184/metrics | grep pruner_current_phase'
```

## 6. Validation and Expected Results

### 6.1 Success Criteria

A successful test run should meet the following criteria:

- ✅ **Metrics Endpoint Accessible**: `http://localhost:9184/metrics` returns valid Prometheus data
- ✅ **Pruner Phases Active**: `pruner_current_phase` shows valid phase values (0/1/2)
- ✅ **Nodes Deleted**: Either `pruner_sweep_nodes_deleted` or `pruner_incremental_sweep` shows deletions
- ✅ **Space Reclaimed**: `pruner_disk_space_reclaimed_bytes` > 0
- ✅ **No Errors**: `pruner_error_count` = 0
- ✅ **Bloom Filter Initialized**: `pruner_bloom_filter_size_bytes` > 0

### 6.2 Expected Metric Ranges

| Scenario | Expected Nodes Deleted | Expected Space Reclaimed | Duration |
|----------|----------------------|------------------------|----------|
| Counter (500 iterations) | 100-300 nodes | 50-200 KB | 2-3 minutes |
| Object Lifecycle (100 objects) | 50-150 nodes | 100-500 KB | 3-4 minutes |
| Mixed Workload | 200-500 nodes | 200-800 KB | 5-7 minutes |

### 6.3 Sample Test Output

```
=== Rooch Pruner E2E Test Report ===
Timestamp: 2024-01-15T14:30:00.000Z
Duration: 245.7s

Transactions:
  Counter: 500
  Objects Created: 100
  Objects Deleted: 0

Pruner Metrics:
  Current Phase: 2
  Nodes Deleted (Sweep): 145
  Nodes Deleted (Incremental): 287
  Disk Space Reclaimed: 432.15 MB
  Bloom Filter Size: 128.00 MB
  Errors: 0

Validation:
  ✅ prunerStarted
  ✅ incrementalSweepActive
  ✅ nodesDeleted
  ✅ spaceReclaimed
  ✅ noErrors

Overall: ✅ PASSED
```

## 7. Troubleshooting

### 7.1 Common Issues

#### Metrics Not Available

**Symptoms**: `curl http://localhost:9184/metrics` returns 404 or empty response

**Solutions**:
1. Verify pruner metrics are enabled in `run_start_server`
2. Check Prometheus server startup logs
3. Ensure port 9184 is not blocked

#### Pruner Not Running

**Symptoms**: `pruner_current_phase` metric missing or always 0

**Solutions**:
1. Verify pruner configuration: `--pruner-enable true`
2. Check pruner logs for initialization errors
3. Ensure sufficient disk space for bloom filter

#### No Nodes Being Deleted

**Symptoms**: `pruner_sweep_nodes_deleted` and `pruner_incremental_sweep` remain 0

**Solutions**:
1. Wait longer for pruner cycles (default 30 seconds)
2. Check `--pruner-window-days` setting (0 for immediate expiration)
3. Verify transactions are actually creating state changes

### 7.2 Debug Commands

```bash
# Check pruner logs
tail -f ~/.rooch/logs/rooch.log | grep pruner

# Monitor database size
watch -n 10 'du -sh ~/.rooch/data'

# Check RocksDB properties
rooch db inspect --property rocksdb.total-sst-files-size

# View current pruner phase
curl -s http://localhost:9184/metrics | grep pruner_current_phase
```

### 7.3 Performance Issues

#### Slow Test Execution

- Reduce transaction count in test scenarios
- Increase pruner interval to reduce CPU load
- Use smaller bloom filter size

#### High Memory Usage

- Reduce `--pruner-bloom-bits` setting
- Monitor system resources during test execution
- Consider using SSD storage for better performance

## 8. Advanced Topics

### 8.1 Custom Test Scenarios

Extend the test framework for specific use cases:

```typescript
describe('Custom Scenario: High Frequency Updates', () => {
  it('should handle rapid state changes', async () => {
    // Custom test logic for high-frequency scenarios
    const updatesPerSecond = 10
    // ... implementation
  })
})
```

### 8.2 Performance Benchmarking

Create performance benchmarks:

```typescript
describe('Performance Benchmarks', () => {
  it('should measure pruner throughput', async () => {
    const startTime = Date.now()
    // Run large-scale test
    const endTime = Date.now()

    const throughput = totalTransactions / ((endTime - startTime) / 1000)
    console.log(`Throughput: ${throughput} tx/s`)
  })
})
```

### 8.3 Integration with CI/CD

Add pruner tests to CI pipeline:

```yaml
# .github/workflows/pruner-tests.yml
name: Pruner E2E Tests
on: [push, pull_request]
jobs:
  pruner-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Pruner E2E Tests
        run: |
          make build
          cd sdk/typescript/test-suite
          pnpm test pruner-e2e
```

### 8.4 Monitoring and Alerting

Set up monitoring for production pruner health:

```prometheus
# Alert if pruner hasn't run recently
ALERT PrunerNotRunning
  IF pruner_current_phase == 0 FOR 5m
  LABELS { severity = "warning" }

# Alert if pruner errors are increasing
ALERT PrunerErrors
  IF increase(pruner_error_count[5m]) > 0
  LABELS { severity = "error" }
```

## 9. Conclusion

This E2E testing framework provides comprehensive coverage of Rooch pruner functionality. By leveraging Prometheus metrics and the TypeScript SDK's testing infrastructure, developers can:

- Validate pruner behavior under various workloads
- Monitor performance and effectiveness
- Catch regressions early in the development cycle
- Document expected behavior for future reference

The framework is designed to be extensible, allowing teams to add custom test scenarios and integrate with existing CI/CD pipelines.
