# Rooch Pruner End-to-End Testing Guide

## Overview

This guide covers comprehensive end-to-end testing of the Rooch Pruner functionality, including automated testing, metrics validation, and disk space monitoring.

## Architecture

### Test Infrastructure

The E2E testing framework consists of:

- **TestBox**: Infrastructure for managing Rooch server instances
- **Prometheus Client**: Collects and validates pruner metrics
- **Workload Generators**: Create various test scenarios
- **Disk Monitor**: Tracks actual disk space usage changes

### Pruner Metrics

Key metrics exported by the pruner:

| Metric | Type | Description |
|--------|------|-------------|
| `pruner_current_phase` | Gauge | Current pruning phase (0=BuildReach, 1=SweepExpired, 2=Incremental) |
| `pruner_reachable_nodes_scanned` | Histogram | Nodes scanned during reachability analysis |
| `pruner_sweep_nodes_deleted` | Histogram | Nodes deleted during sweep operations |
| `pruner_disk_space_reclaimed_bytes` | Counter | **Estimated** disk space reclaimed |
| `pruner_bloom_filter_size_bytes` | Gauge | Bloom filter memory usage |
| `pruner_error_count` | Counter | Number of errors encountered |

### Test Package Structure

```
sdk/typescript/rooch-pruner-e2e/
├── src/
│   ├── case/pruner-e2e.spec.ts    # Main test suite
│   └── utils/
│       ├── prometheus-client.ts   # Metrics collection
│       └── test-reporter.ts       # Report generation
├── package.json                   # Test package configuration
└── README.md                      # Quick start guide
```

## Quick Start

### Prerequisites

```bash
# Node.js >= 18, pnpm >= 9
npm install pnpm@9.10.0 -g

# Built Rooch binary
cargo build --profile optci --bin rooch
```

### Run Basic Test

```bash
cd sdk/typescript/rooch-pruner-e2e
pnpm install
pnpm test:pruner
```

### Run Long-term Test

```bash
# 2-hour intensive test
pnpm test:pruner:long-term:heavy
```

## Test Scenarios

### Standard Test (5-10 minutes)

Quick CI test with minimal workload:

- **Counter Operations**: 1 transaction
- **Object Operations**: 1 create operation
- **Pruner Wait**: 60 seconds
- **Validation**: Basic metrics export check

### Heavy Test (15-20 minutes)

More intensive test for CI:

- **Counter Operations**: 100 transactions
- **Object Operations**: 50 create operations
- **Pruner Wait**: 90 seconds
- **Validation**: Full metrics validation

### Long-term Test (1-8 hours)

Extended monitoring tests:

- **Multiple Cycles**: Configurable workload cycles
- **Extended Duration**: 60-480 minutes
- **Phase Monitoring**: Track pruner phase transitions
- **Disk Monitoring**: Real disk usage tracking

## Test Coverage Strategy

End-to-end tests sit on top of a layered test plan. Each layer has a clear goal and should stay lightweight so failures point to the right component:

1. **SMT unit tests** (`moveos/smt/src/tests.rs`)  
   - Directly exercise `SMTree::puts/updates` and verify `stale_indices` contents.  
   - Tests such as `test_stale_indices_correctness` and `test_stale_indices_with_refcount_simulation` ensure new nodes never show up in `stale_indices` and refcounts remain positive.

2. **Store-level integration tests** (`moveos/moveos-store/src/tests`)  
   - Use SMT APIs plus `prune_store` helpers to mimic `handle_tx_output`.  
   - Verify refcount increments/decrements, `write_stale_indices`, and `IncrementalSweep` end-to-end without spinning up the full pruner loop.

3. **Pruner E2E suite** (`sdk/typescript/rooch-pruner-e2e`)  
   - Validates the full lifecycle (transactions + pruner phases + metrics).  
   - Runs under aggressive settings (`--pruner-protection-orders 0`) to surface regressions quickly and reports both metrics and disk observations.

Keeping all three layers green gives confidence that SMT logic, MoveOS store plumbing, and the long-running pruner daemon stay in sync.

## Disk Space Monitoring

### The Measurement Challenge

**Important**: Simple "before/after" disk monitoring is **incorrect** for pruner testing because:

1. Tests continuously write data while pruner deletes old data
2. Net result is often "increased disk usage" (write volume > delete volume)
3. Cannot directly measure pruner effectiveness

### Correct Measurement Strategies

#### Peak-to-Final Difference (Recommended)

Monitor disk usage throughout the test, calculate: `Peak Usage - Final Usage = Reclaimed`

```bash
# Background monitoring script
while true; do
  SIZE=$(du -sb $DATA_DIR | awk '{print $1}')
  if [ $SIZE -gt $PEAK_SIZE ]; then
    PEAK_SIZE=$SIZE
  fi
  sleep 5
done

# Analysis: Peak - Final = Reclaimed by Pruner
```

#### Comparison Methods

| Method | Accuracy | Implementation | Use Case |
|--------|----------|----------------|----------|
| ❌ Simple Before/After | Low | Easy | **Incorrect** |
| ✅ Peak-to-Final | Medium-High | Medium | **Recommended** |
| Stage Separation | High | Hard | Advanced testing |
| Control Comparison | Highest | Hard | Scientific validation |

### Metrics vs Reality

Current implementation provides both:

- **Estimated Reclaimed** (from metrics): `nodes_deleted × 32_bytes`
- **Actual Reclaimed** (from disk monitoring): Peak-to-final difference

**Expected Results**:
```markdown
Estimated: 195 MB (1M nodes × 32 bytes)
Actual: 200 MB (real disk difference)
Accuracy: 102.6%
```

## GitHub Actions Workflow

### Manual Trigger Workflow

The repository includes a specialized workflow for long-term pruner testing:

```yaml
# .github/workflows/pruner_long_term_test.yml
name: Pruner Long-term Integration Test
on:
  workflow_dispatch:
    inputs:
      test_duration_minutes:
        default: '120'
      workload_intensity:
        options: ['light', 'medium', 'heavy']
      enable_disk_monitoring:
        default: true
```

### Workflow Features

- **Configurable Parameters**: Duration, intensity, monitoring options
- **Real Disk Monitoring**: Peak-to-final disk usage tracking
- **Comprehensive Reporting**: Metrics, disk usage, artifacts
- **Self-hosted Runner**: Suitable for long-running tests

### Generated Artifacts

- `test-output.log`: Complete test execution logs
- `test-report.md`: Formatted test report
- `metrics-snapshot.txt`: Final Prometheus metrics
- `disk-usage-timeline.csv`: Disk usage over time

## Implementation Details

### Current Implementation Status

✅ **Completed**:
- Test package structure (`@roochnetwork/rooch-pruner-e2e`)
- Basic pruner metrics collection
- Standard and heavy test scenarios
- Disk space monitoring (peak-to-final method)
- GitHub Actions workflow
- Comprehensive documentation

🔄 **In Progress**:
- Advanced disk monitoring integration
- Database-level statistics collection
- Performance benchmarking suite

### Key Files

**Test Suite**:
- `sdk/typescript/rooch-pruner-e2e/src/case/pruner-e2e.spec.ts`

**Utilities**:
- `sdk/typescript/rooch-pruner-e2e/src/utils/prometheus-client.ts`
- `sdk/typescript/rooch-pruner-e2e/src/utils/test-reporter.ts`

**Configuration**:
- `sdk/typescript/rooch-pruner-e2e/package.json`
- `sdk/typescript/rooch-pruner-e2e/vitest.config.ts`

**Workflow**:
- `.github/workflows/pruner_long_term_test.yml`

## Troubleshooting

### Common Issues

#### Metrics Not Available
```bash
# Check if metrics server is running
curl http://localhost:9184/metrics | grep pruner

# Verify pruner is enabled
rooch server start --help | grep pruner
```

#### Tests Timeout
- Increase `PRUNER_SETTLE_MS` for slower systems
- Check system resources (CPU, memory)
- Verify pruner configuration is correct

#### Disk Monitoring Issues
- Ensure test data directory has sufficient space
- Check file system supports `du` command
- Verify background monitoring script permissions

### Debug Commands

```bash
# Monitor pruner metrics in real-time
watch -n 5 'curl -s http://localhost:9184/metrics | grep pruner_current_phase'

# Check database size changes
watch -n 10 'du -sh ~/.rooch/data'

# View test logs
tail -f sdk/typescript/rooch-pruner-e2e/test-output.log
```

## Future Enhancements

### Planned Features

1. **Database Statistics Integration**
   - Direct RocksDB metrics collection
   - Accurate disk space calculations
   - Compression-aware monitoring

2. **Advanced Performance Analysis**
   - Pruner throughput benchmarking
   - Memory usage profiling
   - I/O pattern analysis

3. **Distributed Testing**
   - Multi-node pruner coordination
   - Network partition testing
   - Recovery scenario validation

### Contributing

See the main project documentation for contribution guidelines. For pruner-specific testing:

1. Add new test scenarios to `pruner-e2e.spec.ts`
2. Update metrics validation in `prometheus-client.ts`
3. Add corresponding documentation
4. Test with both standard and long-term scenarios

---

## References

- [Rooch Pruner Architecture](../../crates/rooch-pruner/)
- [TestBox Infrastructure](../../sdk/typescript/test-suite/src/testbox.ts)
- [GitHub Actions Workflow](../.github/workflows/pruner_long_term_test.yml)
- [Move Contract Examples](../../examples/pruner_test/)