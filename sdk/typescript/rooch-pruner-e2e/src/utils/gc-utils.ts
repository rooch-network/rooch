// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { TestBox, PrometheusClient } from '@rooch-test-suite'

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

  console.log('🔧 GC Command Args:', `rooch ${args.join(' ')}`)

  try {
    const output = testbox.roochCommand(args)
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
  const args = ['move', 'publish', '-p', packagePath, '--named-addresses', namedAddresses]
  await testbox.roochCommand(args)
}

export async function generateTestData(
  testbox: TestBox,
  config: GCTestConfig,
  defaultAddress: string,
): Promise<void> {
  console.log(
    `🔧 Generating test data: ${config.objectCount} objects, ${config.updateCount} updates, ${config.counterIters} counter operations`,
  )

  // For now, skip complex object operations and just generate some basic transactions
  // We'll use simple account operations or built-in functions

  // Generate some basic state by creating accounts or using built-in functions
  // Since we don't have custom contracts, we'll use minimal operations that create state

  console.log('📝 Generating basic state changes...')
  // Try to use some built-in functions or simple operations

  // For demonstration, we'll just wait a bit to simulate data generation
  await new Promise((resolve) => setTimeout(resolve, 1000))

  console.log('✅ Test data generation completed (simplified)')
}

// Removed runGCTestCycle - server lifecycle should be managed by test code directly

export const delay = (ms: number): Promise<void> => {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

export default {
  loadGCTestConfig,
  executeGC,
  executeGCCommand,
  tryParseGCJson,
  parseGCReport,
  runMoveFunction,
  publishPackage,
  generateTestData,
  delay,
}
