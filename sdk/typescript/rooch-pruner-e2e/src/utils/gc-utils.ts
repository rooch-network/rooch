// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { TestBox } from '@roochnetwork/test-suite'

// GC test configuration
export interface GCTestConfig {
  // Data generation parameters
  objectCount: number
  updateCount: number
  deleteCount: number
  counterIters: number

  // GC command parameters
  dryRun: boolean
  batchSize: number
  workers: number
  markerStrategy: 'auto' | 'memory' | 'persistent'
  useRecycleBin: boolean
  protectedRootsCount: number
  skipConfirm: boolean
  verbose: boolean
  json: boolean

  // Test parameters
  settleMs: number
  testTimeout: number
}

// GC JSON report structure (matches gc.rs JsonReport)
export interface GCJsonReport {
  executionMode: 'dry-run' | 'execute'
  protectedRoots: {
    count: number
    roots: string[]
  }
  markStats: {
    markedCount: number
    durationMs: number
    memoryStrategy: string
  }
  sweepStats: {
    scannedCount: number
    keptCount: number
    deletedCount: number
    recycleBinEntries: number
    durationMs: number
  }
  memoryStrategyUsed: string
  durationMs: number
  spaceReclaimed: number
}

// GC report structure (legacy, for backward compatibility)
export interface GCReport {
  mode: 'dry-run' | 'execution'
  startTime: string
  endTime?: string
  config: {
    batchSize: number
    workers: number
    markerStrategy: string
    useRecycleBin: boolean
    protectedRootsCount: number
    forceCompaction: boolean
  }
  results: {
    phase: string
    success: boolean
    error?: string
    metrics?: {
      totalNodes: number
      visitedNodes: number
      reachableNodes: number
      deletedNodes: number
      executionTime: number
    }
  }[]
  summary: {
    totalNodes: number
    reachableNodes: number
    deletedNodes: number
    executionTime: number
  }
}

// GC execution result
export interface GCResult {
  success: boolean
  output: string
  report?: GCJsonReport // Only present if JSON parsing succeeds
}

// Stress test configuration for long-running tests
export interface GCStressTestConfig {
  enabled: boolean // Enable stress test mode
  durationSeconds: number // How long to run the test (seconds)
  tps: number // Target transactions per second
  batchSize: number // Number of operations per batch
  reportIntervalSeconds: number // How often to report progress (seconds)
  mixRatio: {
    // Operation mix ratio (should sum to 1.0)
    create: number // Create new objects
    update: number // Update existing objects
    delete: number // Delete objects
    counter: number // Counter operations
  }
}

export function loadGCStressTestConfig(): GCStressTestConfig {
  const enabled = process.env.GC_STRESS_MODE === 'true'
  const durationSeconds = parseInt(process.env.GC_STRESS_DURATION || '3600', 10) // Default: 1 hour
  const tps = parseFloat(process.env.GC_STRESS_TPS || '10') // Default: 10 tx/sec

  return {
    enabled,
    durationSeconds,
    tps,
    batchSize: parseInt(process.env.GC_STRESS_BATCH_SIZE || '10', 10),
    reportIntervalSeconds: parseInt(process.env.GC_STRESS_REPORT_INTERVAL || '60', 10), // Default: 1 min
    mixRatio: {
      create: parseFloat(process.env.GC_STRESS_MIX_CREATE || '0.4'), // 40% create
      update: parseFloat(process.env.GC_STRESS_MIX_UPDATE || '0.3'), // 30% update
      delete: parseFloat(process.env.GC_STRESS_MIX_DELETE || '0.2'), // 20% delete
      counter: parseFloat(process.env.GC_STRESS_MIX_COUNTER || '0.1'), // 10% counter
    },
  }
}

export function loadGCTestConfig(): GCTestConfig {
  return {
    // Data generation - optimized for faster tests
    objectCount: parseInt(process.env.GC_OBJECT_COUNT || '20', 10),
    updateCount: parseInt(process.env.GC_UPDATE_COUNT || '10', 10),
    deleteCount: parseInt(process.env.GC_DELETE_COUNT || '15', 10),
    counterIters: parseInt(process.env.GC_COUNTER_ITERS || '5', 10),

    // GC command parameters
    dryRun: process.env.GC_DRY_RUN !== 'false',
    batchSize: parseInt(process.env.GC_BATCH_SIZE || '10000', 10),
    workers: parseInt(process.env.GC_WORKERS || '4', 10),
    markerStrategy: (process.env.GC_MARKER_STRATEGY as 'auto' | 'memory' | 'persistent') || 'auto',
    useRecycleBin: process.env.GC_RECYCLE_BIN !== 'false',
    protectedRootsCount: parseInt(process.env.GC_PROTECTED_ROOTS_COUNT || '1', 10),
    skipConfirm: process.env.GC_SKIP_CONFIRM === 'true' || process.env.GC_FORCE === 'true',
    verbose: process.env.GC_VERBOSE === 'true', // default to quiet to keep output small for CI
    json: process.env.GC_JSON === 'true',

    // Test parameters - optimized for speed
    settleMs: parseInt(process.env.GC_SETTLE_MS || '5000', 10), // Reduced from 30s
    testTimeout: parseInt(process.env.GC_TEST_TIMEOUT || '120000', 10), // 2 minutes instead of 10
  }
}

export function executeGC(testbox: TestBox, options: { dryRun?: boolean }): GCResult {
  // The data directory path should match what the server uses
  const dataDir = `${testbox.roochDir}/data`

  const args = [
    'db',
    'gc',
    '--chain-id',
    'local', // Must match the network name used when starting the server
    '--data-dir',
    dataDir, // Must match the data directory used by the server
    ...(options.dryRun ? ['--dry-run'] : ['--skip-confirm']),
    '--recycle-bin',
    '--json', // Request JSON output but don't depend on it
  ]

  // Set RUST_LOG=error to suppress info logs that break JSON output
  const envs = ['RUST_LOG=error']

  console.log('🔧 GC Command Args:', `RUST_LOG=error rooch ${args.join(' ')}`)

  try {
    const output = testbox.roochCommand(args, envs)
    // Try to parse JSON, but don't fail if parsing fails
    const report = tryParseGCJson(output)
    return { success: true, output, report }
  } catch (error: any) {
    // Command failed
    return { success: false, output: error?.message || String(error) }
  }
}

// Legacy function for backward compatibility
export async function executeGCCommand(testbox: TestBox, options: GCTestConfig): Promise<string> {
  const result = executeGC(testbox, { dryRun: options.dryRun })
  if (!result.success) {
    throw new Error(result.output)
  }
  return result.output
}

// Try to parse GC JSON output, return undefined if parsing fails
export function tryParseGCJson(output: string): GCJsonReport | undefined {
  try {
    // Look for JSON object in the output
    const jsonMatch = output.match(/\{[\s\S]*\}/)
    if (jsonMatch) {
      const report = JSON.parse(jsonMatch[0])

      // Validate required fields
      if (
        typeof report.executionMode === 'string' &&
        typeof report.markStats === 'object' &&
        typeof report.sweepStats === 'object'
      ) {
        return report
      }
    }
  } catch {
    // Parsing failed, return undefined
  }
  return undefined
}

// Print GC report in a formatted way
export function printGCReport(report: GCJsonReport): void {
  console.log('\n📊 GC Report:')
  console.log('==========================================')
  console.log(`Execution Mode: ${report.executionMode}`)
  console.log(`Protected Roots: ${report.protectedRoots.count}`)
  console.log(`Mark Phase:`)
  console.log(`  - Nodes Marked: ${report.markStats.markedCount}`)
  console.log(`  - Duration: ${report.markStats.durationMs}ms`)
  console.log(`  - Strategy: ${report.markStats.memoryStrategy}`)
  console.log(`Sweep Phase:`)
  console.log(`  - Scanned: ${report.sweepStats.scannedCount}`)
  console.log(`  - Kept: ${report.sweepStats.keptCount}`)
  console.log(`  - Deleted: ${report.sweepStats.deletedCount}`)
  console.log(`  - Recycle Bin Entries: ${report.sweepStats.recycleBinEntries}`)
  console.log(`  - Duration: ${report.sweepStats.durationMs}ms`)
  console.log(`Summary:`)
  console.log(`  - Memory Strategy: ${report.memoryStrategyUsed}`)
  console.log(`  - Total Duration: ${report.durationMs}ms`)
  console.log(`  - Space Reclaimed: ${report.spaceReclaimed.toFixed(2)}%`)
  console.log('==========================================\n')
}

// Legacy parse function for backward compatibility
export function parseGCReport(
  output: string,
  isDryRun: boolean,
  isJson: boolean = false,
): GCReport {
  try {
    let jsonOutput = output.trim()

    // Handle mixed output where JSON might be at the end
    if (!isJson && jsonOutput.includes('\n')) {
      // Look for JSON at the end of the output
      const jsonMatch = jsonOutput.match(/\{[\s\S]*\}$/)
      if (jsonMatch) {
        jsonOutput = jsonMatch[0]
      }
    }

    const report = JSON.parse(jsonOutput)

    // Add metadata
    report.mode = isDryRun ? 'dry-run' : 'execution'

    if (!report.config) {
      report.config = {} as any
    }

    if (!report.results) {
      report.results = []
    }

    if (!report.summary) {
      report.summary = {} as any
    }

    return report
  } catch (error) {
    // Enhanced error handling for JSON parsing
    const errorMessage = error instanceof Error ? error.message : String(error)

    // Try to extract JSON from the end of the output
    if (output.trim() && !output.trim().startsWith('{')) {
      const lines = output.trim().split('\n')
      for (let i = lines.length - 1; i >= 0; i--) {
        if (lines[i].trim().startsWith('{')) {
          try {
            const jsonStr = lines.slice(i).join('\n')
            const report = JSON.parse(jsonStr)
            report.mode = isDryRun ? 'dry-run' : 'execution'
            if (!report.config) report.config = {} as any
            if (!report.results) report.results = []
            if (!report.summary) report.summary = {} as any
            return report
          } catch (e) {
            // Continue trying
          }
        }
      }
    }

    throw new Error(`Failed to parse GC report: ${errorMessage}`)
  }
}

export async function runMoveFunction(
  testbox: TestBox,
  functionId: string,
  args: string[],
): Promise<any> {
  const command = [
    'move',
    'run',
    '--function',
    functionId,
    '--config-dir',
    testbox.roochDir,
    ...args.flatMap((arg) => ['--args', arg]),
  ]

  const output = await testbox.roochCommand(command)
  console.log('Move function output:', output)

  // Most Move functions return human-readable output, not JSON
  // Only try to parse JSON if the output actually looks like JSON
  const trimmedOutput = output.trim()
  if (trimmedOutput.startsWith('{') && trimmedOutput.endsWith('}')) {
    try {
      return JSON.parse(trimmedOutput)
    } catch (parseError) {
      console.warn('Failed to parse JSON from Move function output:', parseError)
    }
  }

  // For non-JSON output, just return a success indicator
  return { success: true, output }
}

export async function publishPackage(
  testbox: TestBox,
  packagePath: string,
  namedAddresses: string,
): Promise<void> {
  console.log(`📦 Publishing package at ${packagePath}`)
  const ok = await testbox.cmdPublishPackage(packagePath, { namedAddresses })
  if (!ok) {
    throw new Error(`Failed to publish package at ${packagePath}`)
  }
  console.log(`✅ Successfully published package`)
}

export async function generateTestData(
  testbox: TestBox,
  config: GCTestConfig,
  defaultAddress: string,
): Promise<void> {
  console.log(
    `🔧 Generating test data: ${config.objectCount} objects, ${config.updateCount} updates, ${config.counterIters} counter operations`,
  )

  // Use deterministic seeds so object IDs are known
  const seedBase = Date.now() % 1_000_000
  const seeds: number[] = []

  // Counter operations - create more versions
  console.log(`  📝 Counter operations: ${config.counterIters}`)
  for (let i = 0; i < config.counterIters; i++) {
    await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
    if (i % 10 === 0) await delay(10)
  }

  // Object creation
  console.log(`  ➕ Create operations: ${config.objectCount}`)
  for (let i = 0; i < config.objectCount; i++) {
    const seed = seedBase + i
    seeds.push(seed)
    await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::create_named`, [
      `u64:${seed}`,
      `u64:${i * 100}`,
    ])
    if (i % 20 === 0) await delay(5)
  }

  // Update operations
  console.log(`  ✏️  Update operations: ${config.updateCount}`)
  for (let i = 0; i < config.updateCount && i < seeds.length; i++) {
    await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::update_named`, [
      `u64:${seeds[i]}`,
      `u64:${i * 200}`,
    ])
    if (i % 15 === 0) await delay(5)
  }

  // Delete operations
  console.log(`  🗑️  Delete operations: ${config.deleteCount}`)
  for (let i = 0; i < config.deleteCount && i < seeds.length; i++) {
    await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::remove_named`, [
      `u64:${seeds[seeds.length - 1 - i]}`,
    ])
    if (i % 15 === 0) await delay(5)
  }

  console.log('✅ Test data generation completed')
}

// Generate test data continuously for stress testing
export async function generateContinuousData(
  testbox: TestBox,
  config: GCStressTestConfig,
  defaultAddress: string,
  stopSignal: { stop: boolean }, // Shared object to signal stop
): Promise<{
  totalTxs: number
  byType: { create: number; update: number; delete: number; counter: number }
}> {
  const stats = {
    totalTxs: 0,
    byType: { create: 0, update: 0, delete: 0, counter: 0 },
  }

  const startTime = Date.now()
  const endTime = startTime + config.durationSeconds * 1000
  let lastReportTime = startTime
  const createdSeeds: number[] = []
  let seedCounter = Date.now() % 1_000_000

  console.log(`🚀 Starting continuous data generation for ${config.durationSeconds}s`)
  console.log(`   Target TPS: ${config.tps}`)
  console.log(`   Batch Size: ${config.batchSize}`)
  console.log(
    `   Mix Ratio: Create ${(config.mixRatio.create * 100).toFixed(0)}%, Update ${(config.mixRatio.update * 100).toFixed(0)}%, Delete ${(config.mixRatio.delete * 100).toFixed(0)}%, Counter ${(config.mixRatio.counter * 100).toFixed(0)}%`,
  )

  while (Date.now() < endTime && !stopSignal.stop) {
    const batchStartTime = Date.now()

    // Execute a batch of transactions
    for (let i = 0; i < config.batchSize && !stopSignal.stop; i++) {
      const rand = Math.random()
      let cumulativeProbability = 0

      try {
        // Determine operation type based on mix ratio
        if (rand < (cumulativeProbability += config.mixRatio.create)) {
          // Create operation
          const seed = seedCounter++
          createdSeeds.push(seed)
          await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::create_named`, [
            `u64:${seed}`,
            `u64:${Math.floor(Math.random() * 1000)}`,
          ])
          stats.byType.create++
        } else if (rand < (cumulativeProbability += config.mixRatio.update)) {
          // Update operation (only if we have created objects)
          if (createdSeeds.length > 0) {
            const randomIndex = Math.floor(Math.random() * createdSeeds.length)
            const seed = createdSeeds[randomIndex]
            await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::update_named`, [
              `u64:${seed}`,
              `u64:${Math.floor(Math.random() * 1000)}`,
            ])
            stats.byType.update++
          } else {
            // Fallback to counter if no objects to update
            await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
            stats.byType.counter++
          }
        } else if (rand < (cumulativeProbability += config.mixRatio.delete)) {
          // Delete operation (only if we have created objects)
          if (createdSeeds.length > 0) {
            const randomIndex = Math.floor(Math.random() * createdSeeds.length)
            const seed = createdSeeds.splice(randomIndex, 1)[0]
            await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::remove_named`, [
              `u64:${seed}`,
            ])
            stats.byType.delete++
          } else {
            // Fallback to counter if no objects to delete
            await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
            stats.byType.counter++
          }
        } else {
          // Counter operation
          await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
          stats.byType.counter++
        }

        stats.totalTxs++
      } catch (error) {
        console.error(`⚠️ Transaction failed:`, error.message)
        // Continue with next transaction
      }
    }

    // Report progress periodically
    const now = Date.now()
    if (now - lastReportTime >= config.reportIntervalSeconds * 1000) {
      const elapsedSeconds = (now - startTime) / 1000
      const remainingSeconds = (endTime - now) / 1000
      const actualTPS = stats.totalTxs / elapsedSeconds

      console.log(
        `\n📊 Progress Report (${elapsedSeconds.toFixed(0)}s elapsed, ${remainingSeconds.toFixed(0)}s remaining):`,
      )
      console.log(`   Total Transactions: ${stats.totalTxs}`)
      console.log(`   Actual TPS: ${actualTPS.toFixed(2)}`)
      console.log(`   Created: ${stats.byType.create}, Updated: ${stats.byType.update}`)
      console.log(`   Deleted: ${stats.byType.delete}, Counter: ${stats.byType.counter}`)
      console.log(`   Active Objects: ${createdSeeds.length}`)

      lastReportTime = now
    }

    // Calculate delay to achieve target TPS
    const batchDuration = Date.now() - batchStartTime
    const targetBatchDuration = (config.batchSize / config.tps) * 1000
    const delayMs = Math.max(0, targetBatchDuration - batchDuration)

    if (delayMs > 0) {
      await delay(delayMs)
    }
  }

  const totalDuration = (Date.now() - startTime) / 1000
  const actualTPS = stats.totalTxs / totalDuration

  console.log(`\n✅ Continuous data generation completed:`)
  console.log(`   Duration: ${totalDuration.toFixed(2)}s`)
  console.log(`   Total Transactions: ${stats.totalTxs}`)
  console.log(`   Actual TPS: ${actualTPS.toFixed(2)}`)
  console.log(`   Create: ${stats.byType.create}, Update: ${stats.byType.update}`)
  console.log(`   Delete: ${stats.byType.delete}, Counter: ${stats.byType.counter}`)

  return stats
}

// Removed runGCTestCycle - server lifecycle should be managed by test code directly

export const delay = (ms: number): Promise<void> => {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

const gcUtils = {
  loadGCTestConfig,
  loadGCStressTestConfig,
  executeGC,
  executeGCCommand,
  tryParseGCJson,
  printGCReport,
  parseGCReport,
  runMoveFunction,
  publishPackage,
  generateTestData,
  generateContinuousData,
  delay,
}

export default gcUtils
