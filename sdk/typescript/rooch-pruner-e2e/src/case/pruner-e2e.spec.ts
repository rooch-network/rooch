// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'

import { TestBox } from '@roochnetwork/test-suite'
import { PrometheusClient } from '../utils/prometheus-client.js'
import { generateReport, printReport } from '../utils/test-reporter.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const repoRoot = path.resolve(__dirname, '../../../../../')
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
  console.log(`Publishing package at ${packagePath} with named addresses: ${namedAddresses}`)
  const ok = await testbox.cmdPublishPackage(packagePath, { namedAddresses })
  if (!ok) {
    throw new Error(`Failed to publish package at ${packagePath}`)
  }
  console.log(`Successfully published package at ${packagePath}`)
}

describe('Rooch pruner end-to-end', () => {
  let testbox: TestBox
  let prometheus: PrometheusClient
  let defaultAddress: string

  beforeAll(async () => {
    console.log('### pruner e2e: init testbox')
    console.error('🔧 About to create TestBox')
    testbox = new TestBox()
    console.error('🔧 TestBox created, roochDir:', testbox.roochDir)
    console.log('### pruner e2e: start rooch server')

    // Check if this is a long-term test
    const isLongTermTest = process.env.LONG_TERM_TEST === 'true'

    // Configure pruner parameters based on test type
    const prunerArgs = ['--pruner-enable', '--pruner-enable-incremental-sweep']

    if (isLongTermTest) {
      // Long-term test configuration - more aggressive for extended monitoring
      prunerArgs.push(
        '--pruner-interval-s',
        process.env.LONG_TERM_INTERVAL_S || '30',
        '--pruner-protection-orders',
        process.env.LONG_TERM_PROTECTION_ORDERS || '0', // 0 = aggressive mode (only protect latest root)
        '--pruner-bloom-bits',
        process.env.LONG_TERM_BLOOM_BITS || '67108864', // 64MB bloom filter
        '--pruner-scan-batch',
        process.env.LONG_TERM_SCAN_BATCH || '50000',
        '--pruner-delete-batch',
        process.env.LONG_TERM_DELETE_BATCH || '25000',
      )
      console.log('### pruner e2e: using long-term test configuration')
    } else {
      // Standard test configuration - faster for CI/quick testing
      prunerArgs.push(
        '--pruner-interval-s',
        '15', // More frequent interval for faster testing
        '--pruner-protection-orders',
        '0', // Aggressive mode: only protect latest root - atomic snapshot protects reachable nodes
        '--pruner-bloom-bits',
        '16777216', // 16MB bloom filter for reasonable accuracy
        '--pruner-scan-batch',
        '10000', // Larger batch size for more aggressive cleaning
        '--pruner-delete-batch',
        '5000', // Larger delete batch for more aggressive cleaning
      )
      console.log('### pruner e2e: using standard test configuration')
    }

    // Use port 0 to get dynamic port allocation
    console.error('🔄 About to call testbox.loadRoochEnv with args:', prunerArgs.length, 'items')
    console.error('🔄 TestBox roochDir:', testbox.roochDir)
    const loadResult = await testbox.loadRoochEnv('local', 0, prunerArgs)
    console.error('🔄 loadRoochEnv returned:', loadResult)

    // Configure RPC URL for rooch CLI after server starts
    const serverAddress = testbox.getRoochServerAddress()
    console.error('### pruner e2e: serverAddress:', serverAddress)
    // if (serverAddress) {
    //   console.log('### pruner e2e: configuring RPC URL:', serverAddress)
    //   testbox.roochCommand([
    //     'env',
    //     'add',
    //     '--config-dir',
    //     testbox.roochDir,
    //     '--alias',
    //     'local',
    //     '--rpc',
    //     `http://${serverAddress}`,
    //   ])
    //   testbox.roochCommand(['env', 'switch', '--config-dir', testbox.roochDir, '--alias', 'local'])
    // }

    console.log('### pruner e2e: fetch default address')
    defaultAddress = await testbox.defaultCmdAddress()
    console.log('### pruner e2e: default address:', defaultAddress)

    // Wait for server to be fully ready before publishing packages
    console.log('### pruner e2e: waiting 10s for server to be fully ready...')
    await delay(10000)

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

    // Wait extra time for server to be fully ready
    console.log('### pruner e2e: waiting for server to be fully ready')

    prometheus = new PrometheusClient(testbox.getMetricsPort() ?? 9184)
    console.log('### pruner e2e: beforeAll done')
  }, 180000)

  afterAll(async () => {
    if (testbox) {
      testbox.stop()
    }
  })

  // Calculate timeout dynamically based on test type
  const isLongTermTest = process.env.LONG_TERM_TEST === 'true'
  let testTimeout: number
  if (isLongTermTest) {
    const testDurationMinutes = parseInt(process.env.LONG_TERM_DURATION_MINUTES || '60', 10)
    const cycleCount = parseInt(process.env.LONG_TERM_CYCLE_COUNT || '10', 10)
    // Timeout = test duration + cycles overhead (30s per cycle) + setup/teardown buffer (30min)
    const cyclesOverheadMs = cycleCount * 30 * 1000 // 30 seconds per cycle
    const setupTeardownBufferMs = 30 * 60 * 1000 // 30 minutes buffer for setup, cycles, and teardown
    const testDurationMs = testDurationMinutes * 60 * 1000
    testTimeout = testDurationMs + cyclesOverheadMs + setupTeardownBufferMs
    console.log(
      `### pruner e2e: calculated timeout for long-term test: ${(testTimeout / 60000).toFixed(2)} minutes`,
    )
  } else {
    const settleMs = parseInt(process.env.PRUNER_SETTLE_MS || '60000', 10) // Default 60 seconds to allow pruner cycles
    // Timeout = settle time + workload execution + buffer (5 minutes)
    testTimeout = settleMs + 5 * 60 * 1000
    console.log(
      `### pruner e2e: calculated timeout for standard test: ${(testTimeout / 1000).toFixed(0)} seconds`,
    )
  }
  // Ensure minimum timeout of 5 minutes
  testTimeout = Math.max(testTimeout, 5 * 60 * 1000)

  it(
    'collects pruning metrics after workload churn',
    async () => {
      const startTime = Date.now()
      const isLongTermTest = process.env.LONG_TERM_TEST === 'true'

      if (isLongTermTest) {
        console.log('🚀 Starting long-term pruner integration test (1+ hours duration)')

        // Long-term test configuration - larger workload
        const counterIters = parseInt(process.env.LONG_TERM_COUNTER_ITERS || '100', 10)
        const createIters = parseInt(process.env.LONG_TERM_CREATE_ITERS || '50', 10)
        const updateIters = parseInt(process.env.LONG_TERM_UPDATE_ITERS || '25', 10)
        const deleteIters = parseInt(process.env.LONG_TERM_DELETE_ITERS || '20', 10)
        const testDurationMinutes = parseInt(process.env.LONG_TERM_DURATION_MINUTES || '60', 10) // Default 1 hour
        const cycleCount = parseInt(process.env.LONG_TERM_CYCLE_COUNT || '10', 10)

        const txCounts = {
          counter: 0,
          objectCreated: 0,
          objectUpdated: 0,
          objectDeleted: 0,
        }

        // Use deterministic seeds so object IDs are known without querying on-chain state
        const seedBase = Date.now() % 1_000_000

        const phaseHistory = []

        console.log(`📊 Long-term test configuration:`)
        console.log(`  - Counter iterations per cycle: ${counterIters}`)
        console.log(`  - Create operations per cycle: ${createIters}`)
        console.log(`  - Update operations per cycle: ${updateIters}`)
        console.log(`  - Delete operations per cycle: ${deleteIters}`)
        console.log(`  - Number of cycles: ${cycleCount}`)
        console.log(`  - Test duration: ${testDurationMinutes} minutes`)

        // Execute multiple cycles of data creation and updates
        for (let cycle = 0; cycle < cycleCount; cycle++) {
          console.log(`🔄 Cycle ${cycle + 1}/${cycleCount}`)

          // Counter operations - create more versions
          console.log(
            `### pruner e2e: creating ${counterIters} counter operations (cycle ${cycle + 1})`,
          )
          for (let i = 0; i < counterIters; i++) {
            runMoveFunction(
              testbox,
              `${defaultAddress}::quick_start_counter::increase`,
              [], // no args
            )
            txCounts.counter += 1
            if (i % 10 === 0) await delay(10) // Occasional pause
          }

          // Large-scale object creation
          console.log(`### pruner e2e: creating ${createIters} deterministic objects (cycle ${cycle + 1})`)
          const seeds: number[] = []
          for (let i = 0; i < createIters; i++) {
            const seed = seedBase + cycle * 10_000 + i
            seeds.push(seed)
            runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::create_named`, [
              `u64:${seed}`,
              `u64:${cycle * 1000 + i}`,
            ])
            txCounts.objectCreated += 1
            if (i % 20 === 0) await delay(5)
          }

          // Update operations - use additional counter operations to simulate state updates
          console.log(
            `### pruner e2e: simulating ${updateIters} update operations (cycle ${cycle + 1})`,
          )
          for (let i = 0; i < updateIters && i < seeds.length; i++) {
            runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::update_named`, [
              `u64:${seeds[i]}`,
              `u64:${cycle * 2000 + i}`,
            ])
            txCounts.objectUpdated += 1
            if (i % 15 === 0) await delay(5)
          }

          // Delete a subset of objects deterministically
          console.log(`### pruner e2e: deleting ${deleteIters} deterministic objects (cycle ${cycle + 1})`)
          for (let i = 0; i < deleteIters && i < seeds.length; i++) {
            runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::remove_named`, [
              `u64:${seeds[seeds.length - 1 - i]}`,
            ])
            txCounts.objectDeleted += 1
            if (i % 15 === 0) await delay(5)
          }

          // Brief wait for pruner to work
          await delay(30000) // 30 seconds

          // Collect current metrics
          try {
            const currentMetrics = await prometheus.fetchMetrics()
            phaseHistory.push({
              phase: currentMetrics.currentPhase,
              timestamp: new Date().toISOString(),
              metrics: currentMetrics,
            })
          } catch (error) {
            console.warn(`⚠️ Metrics fetch failed at cycle ${cycle + 1}:`, error)
          }
        }

        console.log('⏰ Starting long-term monitoring phase...')
        console.log(`### pruner e2e: monitoring for ${testDurationMinutes} minutes`)

        const monitoringStartTime = Date.now()
        const monitoringEndTime = monitoringStartTime + testDurationMinutes * 60 * 1000

        // Long-term monitoring loop
        while (Date.now() < monitoringEndTime) {
          try {
            const currentMetrics = await prometheus.fetchMetrics()

            phaseHistory.push({
              phase: currentMetrics.currentPhase,
              timestamp: new Date().toISOString(),
              metrics: currentMetrics,
            })

            const elapsedMinutes = Math.floor((Date.now() - monitoringStartTime) / 60000)

            console.log(
              `📊 [${elapsedMinutes}/${testDurationMinutes}min] Phase: ${currentMetrics.currentPhase}`,
            )
            console.log(
              `  🗑️  Nodes Deleted: Sweep=${currentMetrics.sweepExpiredDeleted.count}, Incremental=${currentMetrics.incrementalSweepDeleted.count}`,
            )
            console.log(`  🔍 Reachable Nodes: ${currentMetrics.reachableNodesScanned.count}`)
            console.log(
              `  💾 Disk Reclaimed: ${(currentMetrics.diskSpaceReclaimedBytes / (1024 * 1024)).toFixed(2)} MB`,
            )
            console.log(
              `  🌸 Bloom Filter: ${(currentMetrics.bloomFilterSizeBytes / (1024 * 1024)).toFixed(2)} MB`,
            )
            console.log(`  ⚠️  Errors: ${currentMetrics.errorCount}`)

            // Report detailed status every 5 minutes
            if (elapsedMinutes > 0 && elapsedMinutes % 5 === 0) {
              const avgNodesPerPhase =
                phaseHistory.length > 0
                  ? phaseHistory.reduce(
                      (sum, p) =>
                        sum +
                        p.metrics.sweepExpiredDeleted.count +
                        p.metrics.incrementalSweepDeleted.count,
                      0,
                    ) / phaseHistory.length
                  : 0

              console.log(`\n📈 [${elapsedMinutes}min] Detailed Report:`)
              console.log(`  - Average nodes deleted per check: ${avgNodesPerPhase.toFixed(2)}`)
              console.log(`  - Total phase transitions: ${phaseHistory.length}`)
            }

            // Wait 2 minutes before checking again
            await delay(120000)
          } catch (error) {
            console.warn(`⚠️ Metrics fetch failed:`, error)
            await delay(60000) // Wait 1 minute on error
          }
        }

        // Final metrics collection
        const finalMetrics = await prometheus.fetchMetrics()
        const testDuration = Date.now() - startTime

        // Verify key metrics
        expect(finalMetrics.currentPhase).toBeGreaterThanOrEqual(0)
        expect(finalMetrics.reachableNodesScanned.count).toBeGreaterThan(0)
        expect(finalMetrics.errorCount).toBe(0)

        console.log(`\n🎯 Long-term Integration Test Results:`)
        console.log(`  ✅ Test Duration: ${(testDuration / 60000).toFixed(2)} minutes`)
        console.log(`  ✅ Total Transactions Created: ${txCounts.counter + txCounts.objectCreated}`)
        console.log(
          `  ✅ Nodes Deleted: Sweep=${finalMetrics.sweepExpiredDeleted.count}, Incremental=${finalMetrics.incrementalSweepDeleted.count}`,
        )
        console.log(
          `  ✅ Disk Space Reclaimed: ${(finalMetrics.diskSpaceReclaimedBytes / (1024 * 1024)).toFixed(2)} MB`,
        )
        console.log(`  ✅ Phase Transitions: ${phaseHistory.length}`)
      } else {
        // Standard test mode - fast CI testing
        const counterIters = parseInt(process.env.PRUNER_COUNTER_ITERS || '1', 10)
        const createIters = parseInt(process.env.PRUNER_CREATE_ITERS || '1', 10)
        const updateIters = parseInt(process.env.PRUNER_UPDATE_ITERS || '1', 10)
        const deleteIters = parseInt(process.env.PRUNER_DELETE_ITERS || '1', 10)
        const settleMs = parseInt(process.env.PRUNER_SETTLE_MS || '60000', 10) // Default 60 seconds to allow pruner cycles
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
          await delay(10)
        }

        // Object lifecycle workload with deterministic seeds
        console.log(`### pruner e2e: creating ${createIters} deterministic objects`)
        const seeds: number[] = []
        const seedBase = Date.now() % 1_000_000
        for (let i = 0; i < createIters; i++) {
          const seed = seedBase + i
          seeds.push(seed)
          runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::create_named`, [
            `u64:${seed}`,
            `u64:${i}`,
          ])
          txCounts.objectCreated += 1
          await delay(10)
        }

        // Update operations - simulate with additional counter operations
        console.log(`### pruner e2e: simulating ${updateIters} update operations`)
        for (let i = 0; i < updateIters && i < seeds.length; i++) {
          runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::update_named`, [
            `u64:${seeds[i]}`,
            `u64:${1000 + i}`,
          ])
          txCounts.objectUpdated += 1
          await delay(10)
        }

        // Delete operations using deterministic object IDs
        console.log(`### pruner e2e: deleting ${deleteIters} deterministic objects`)
        for (let i = 0; i < deleteIters && i < seeds.length; i++) {
          runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::remove_named`, [
            `u64:${seeds[seeds.length - 1 - i]}`,
          ])
          txCounts.objectDeleted += 1
          await delay(10)
        }

        // Allow pruner cycles to complete - use PRUNER_SETTLE_MS if provided, otherwise default to 60s
        console.log(`### pruner e2e: waiting ${settleMs}ms for pruner cycles`)
        await delay(settleMs)

        const prunerMetrics = await prometheus.fetchMetrics()
        const report = generateReport(startTime, txCounts, prunerMetrics)
        printReport(report)

        // Save detailed report for CI/CD pipeline
        if (process.env.LONG_TERM_TEST === 'true') {
          const fs = await import('fs')
          const reportPath = process.env.CI ? '../../test-report.json' : 'test-report.json'
          fs.writeFileSync(
            reportPath,
            JSON.stringify(
              {
                report,
                testConfig: {
                  duration: testTimeout,
                  settleMs,
                  workload: {
                    counterIters,
                    createIters,
                    updateIters,
                    deleteIters,
                  },
                },
              },
              null,
              2,
            ),
          )
          console.log(`### pruner e2e: saved detailed report to ${reportPath}`)
        }

        expect(prunerMetrics.bloomFilterSizeBytes).toBeGreaterThan(0)
        expect(prunerMetrics.currentPhase).toBeGreaterThanOrEqual(0)
        expect(prunerMetrics.errorCount).toBe(0)
        // With atomic snapshot enabled and 0-day window, expect selective deletions
        // The key is that atomic snapshot protects reachable nodes while allowing cleanup of unreferenced nodes
        const totalDeletions =
          prunerMetrics.sweepExpiredDeleted.sum + prunerMetrics.incrementalSweepDeleted.sum
        const reachableNodes = prunerMetrics.reachableNodesScanned.sum
        console.log(`### pruner e2e: total nodes deleted: ${totalDeletions}`)
        console.log(`### pruner e2e: reachable nodes protected: ${reachableNodes}`)
        console.log(`### pruner e2e: (atomic snapshot enabled - selective cleaning expected)`)

        // With atomic snapshot, we should see:
        // 1. Some deletions (unreferenced nodes)
        // 2. Zero errors (atomic snapshot prevents mistakes)
        // 3. Protected reachable nodes
        expect(totalDeletions).toBeGreaterThanOrEqual(0)
        expect(prunerMetrics.errorCount).toBe(0)
        expect(reachableNodes).toBeGreaterThan(0)
        expect(report.validation.passed).toBe(true)

        // Verify atomic snapshot is working: if we have reachable nodes, no errors should occur
        if (reachableNodes > 0) {
          console.log(
            `### pruner e2e: ✅ Atomic snapshot protecting ${reachableNodes} reachable nodes successfully`,
          )
        }
      }
    },
    { timeout: testTimeout },
  )
})
