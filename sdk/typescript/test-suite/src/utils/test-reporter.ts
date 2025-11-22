import { PrunerMetrics } from './prometheus-client.js'

export interface TestReport {
  timestamp: string
  duration: number
  transactions: {
    counter: number
    objectCreated: number
    objectUpdated: number
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
  txCounts: { counter: number; objectCreated: number; objectUpdated: number; objectDeleted: number },
  metrics: PrunerMetrics,
): TestReport {
  const checks = {
    metricsExported: metrics.bloomFilterSizeBytes > 0,
    prunerRan:
      metrics.reachableNodesScanned.count > 0 ||
      metrics.sweepExpiredDeleted.count > 0 ||
      metrics.incrementalSweepDeleted.count > 0,
    deletionsObserved:
      metrics.sweepExpiredDeleted.sum > 0 || metrics.incrementalSweepDeleted.sum > 0,
    noErrors: metrics.errorCount === 0,
  }

  return {
    timestamp: new Date().toISOString(),
    duration: Date.now() - startTime,
    transactions: txCounts,
    prunerMetrics: metrics,
    validation: {
      passed: Object.values(checks).every((v) => v),
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
  console.log(`  Objects Updated: ${report.transactions.objectUpdated}`)
  console.log(`  Objects Deleted: ${report.transactions.objectDeleted}`)
  console.log('\nPruner Metrics:')
  console.log(`  Current Phase: ${report.prunerMetrics.currentPhase}`)
  console.log(
    `  Nodes Deleted (Sweep): count=${report.prunerMetrics.sweepExpiredDeleted.count}, sum=${report.prunerMetrics.sweepExpiredDeleted.sum}`,
  )
  console.log(
    `  Nodes Deleted (Incremental): count=${report.prunerMetrics.incrementalSweepDeleted.count}, sum=${report.prunerMetrics.incrementalSweepDeleted.sum}`,
  )
  console.log(
    `  Reachable Nodes Scanned: count=${report.prunerMetrics.reachableNodesScanned.count}, sum=${report.prunerMetrics.reachableNodesScanned.sum}`,
  )
  console.log(
    `  Disk Space Reclaimed: ${(report.prunerMetrics.diskSpaceReclaimedBytes / 1024 / 1024).toFixed(
      2,
    )} MB`,
  )
  console.log(
    `  Bloom Filter Size: ${(report.prunerMetrics.bloomFilterSizeBytes / 1024 / 1024).toFixed(
      2,
    )} MB`,
  )
  console.log(`  Errors: ${report.prunerMetrics.errorCount}`)
  console.log('\nValidation:')
  for (const [check, passed] of Object.entries(report.validation.checks)) {
    console.log(`  ${passed ? '✅' : '❌'} ${check}`)
  }
  console.log(`\nOverall: ${report.validation.passed ? '✅ PASSED' : '❌ FAILED'}`)
}
