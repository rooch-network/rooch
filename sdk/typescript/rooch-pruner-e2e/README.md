# Rooch Pruner E2E Tests

This package contains end-to-end tests for the Rooch Pruner functionality.

## Overview

The Rooch Pruner is responsible for cleaning up stale state nodes in the Rooch storage system. These E2E tests verify that the pruner correctly:

- Identifies and removes expired state nodes
- Protects reachable nodes using atomic snapshots
- Reports accurate metrics via Prometheus
- Handles various workload patterns (counter operations, object lifecycle, etc.)

## Prerequisites

- Node.js >= 18.0.0
- pnpm >= 9.1.0
- A built Rooch binary (for running the server)

## Running Tests

### Standard Test

Quick test suitable for CI:

```bash
pnpm test:pruner
```

### Heavy Test

More intensive test with larger workloads:

```bash
pnpm test:pruner:heavy
```

### Long-term Test

Extended test for monitoring pruner behavior over time (1+ hours):

```bash
pnpm test:pruner:long-term
```

### Extra Heavy Long-term Test

Maximum intensity test (2+ hours):

```bash
pnpm test:pruner:long-term:heavy
```

## Environment Variables

You can customize test behavior using environment variables:

### Standard Test Variables

- `PRUNER_COUNTER_ITERS`: Number of counter operations (default: 1)
- `PRUNER_CREATE_ITERS`: Number of object creation operations (default: 1)
- `PRUNER_UPDATE_ITERS`: Number of update operations (default: 1)
- `PRUNER_DELETE_ITERS`: Number of delete operations (default: 1)
- `PRUNER_SETTLE_MS`: Time to wait for pruner cycles (default: 60000)

### Long-term Test Variables

- `LONG_TERM_TEST`: Set to `true` to enable long-term test mode
- `LONG_TERM_DURATION_MINUTES`: Test duration in minutes (default: 60)
- `LONG_TERM_COUNTER_ITERS`: Counter operations per cycle (default: 100)
- `LONG_TERM_CREATE_ITERS`: Create operations per cycle (default: 50)
- `LONG_TERM_UPDATE_ITERS`: Update operations per cycle (default: 25)
- `LONG_TERM_DELETE_ITERS`: Delete operations per cycle (default: 20)
- `LONG_TERM_CYCLE_COUNT`: Number of workload cycles (default: 10)
- `LONG_TERM_INTERVAL_S`: Pruner interval in seconds (default: 30)
- `LONG_TERM_PROTECTION_ORDERS`: Number of recent tx_orders to protect (default: 0, aggressive mode)
- `LONG_TERM_BLOOM_BITS`: Bloom filter size in bits (default: 67108864)
- `LONG_TERM_SCAN_BATCH`: Scan batch size (default: 50000)
- `LONG_TERM_DELETE_BATCH`: Delete batch size (default: 25000)

## Architecture

This package depends on `@roochnetwork/test-suite` which provides:

- `TestBox`: Test infrastructure for managing Rooch server instances
- Container management for isolated test environments

The tests use Prometheus metrics exposed by the Rooch server to verify pruner behavior.

## Test Structure

- `src/case/pruner-e2e.spec.ts`: Main test suite
- `src/utils/prometheus-client.ts`: Client for fetching Prometheus metrics
- `src/utils/test-reporter.ts`: Test report generation and formatting
