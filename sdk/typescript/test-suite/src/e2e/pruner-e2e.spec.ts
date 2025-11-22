import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'

import { TestBox } from '../testbox.js'
import { PrometheusClient } from '../utils/prometheus-client.js'
import { generateReport, printReport } from '../utils/test-reporter.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const repoRoot = path.resolve(__dirname, '../../../../..')
const counterPackagePath = path.join(repoRoot, 'examples', 'quick_start_counter')
const prunerPackagePath = path.join(repoRoot, 'examples', 'pruner_test')

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

function runMoveFunction(testbox: TestBox, functionId: string, args: string[]) {
  testbox.roochCommand([
    'move',
    'run',
    '--function',
    functionId,
    ...args.flatMap((arg) => ['--args', arg]),
    '--config-dir',
    testbox.roochDir,
    '--json',
  ])
}

async function publishPackage(testbox: TestBox, packagePath: string, namedAddresses: string) {
  const ok = await testbox.cmdPublishPackage(packagePath, { namedAddresses })
  if (!ok) {
    throw new Error(`Failed to publish package at ${packagePath}`)
  }
}

describe('Rooch pruner end-to-end', () => {
  let testbox: TestBox
  let prometheus: PrometheusClient
  let defaultAddress: string

  beforeAll(async () => {
    console.log('### pruner e2e: init testbox')
    testbox = new TestBox()
    console.log('### pruner e2e: start rooch server')
    // Use fixed port 6767 to match default config used by rooch CLI commands
    await testbox.loadRoochEnv('local', 6767, [
      '--pruner-enable',
      '--pruner-interval-s',
      '15',
      '--pruner-window-days',
      '0',
      '--pruner-enable-incremental-sweep',
      '--pruner-bloom-bits',
      '16777216', // 16MB bloom filter to keep memory reasonable in tests
    ])

    console.log('### pruner e2e: fetch default address')
    defaultAddress = await testbox.defaultCmdAddress()

    console.log('### pruner e2e: publish counter package')
    await publishPackage(
      testbox,
      counterPackagePath,
      'quick_start_counter=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )
    console.log('### pruner e2e: publish pruner package')
    await publishPackage(
      testbox,
      prunerPackagePath,
      'pruner_test=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )

    prometheus = new PrometheusClient(testbox.getMetricsPort() ?? 9184)
    console.log('### pruner e2e: beforeAll done')
  }, 180000)

  afterAll(async () => {
    testbox.stop()
  })

  it(
    'collects pruning metrics after workload churn',
    async () => {
      const startTime = Date.now()
      // Allow load shedding via env to keep memory/time bounded in CI or local runs
      const counterIters = parseInt(process.env.PRUNER_COUNTER_ITERS || '1', 10)
      const createIters = parseInt(process.env.PRUNER_CREATE_ITERS || '1', 10)
      const updateIters = parseInt(process.env.PRUNER_UPDATE_ITERS || '1', 10)
      const deleteIters = parseInt(process.env.PRUNER_DELETE_ITERS || '1', 10)
      const settleMs = parseInt(process.env.PRUNER_SETTLE_MS || '15000', 10)
      const txCounts = {
        counter: 0,
        objectCreated: 0,
        objectUpdated: 0,
        objectDeleted: 0,
      }

      // Counter churn to generate stale versions
      for (let i = 0; i < counterIters; i++) {
        runMoveFunction(
          testbox,
          `${defaultAddress}::quick_start_counter::increase`,
          [], // no args
        )
        txCounts.counter += 1
        await delay(50)
      }

      // Object lifecycle workload
      for (let i = 0; i < createIters; i++) {
        runMoveFunction(
          testbox,
          `${defaultAddress}::object_lifecycle::create_object`,
          [`u64:${i}`, 'u64:1024'],
        )
        txCounts.objectCreated += 1
        await delay(30)
      }

      for (let i = 0; i < updateIters; i++) {
        runMoveFunction(
          testbox,
          `${defaultAddress}::object_lifecycle::update_object`,
          [`u64:${i}`, `u64:${i + 100}`],
        )
        txCounts.objectUpdated += 1
        await delay(30)
      }

      for (let i = 0; i < deleteIters; i++) {
        runMoveFunction(
          testbox,
          `${defaultAddress}::object_lifecycle::remove_object`,
          [`u64:${i}`],
        )
        txCounts.objectDeleted += 1
        await delay(30)
      }

      // Allow at least one full pruning cycle
      await delay(settleMs)

      const prunerMetrics = await prometheus.fetchMetrics()
      const report = generateReport(startTime, txCounts, prunerMetrics)
      printReport(report)

      expect(prunerMetrics.bloomFilterSizeBytes).toBeGreaterThan(0)
      expect(prunerMetrics.currentPhase).toBeGreaterThanOrEqual(0)
      expect(prunerMetrics.errorCount).toBe(0)
      expect(
        prunerMetrics.sweepExpiredDeleted.sum + prunerMetrics.incrementalSweepDeleted.sum,
      ).toBeGreaterThan(0)
      expect(report.validation.passed).toBe(true)
    },
    240000,
  )
})
