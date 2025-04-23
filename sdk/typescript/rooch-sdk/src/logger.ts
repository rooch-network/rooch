// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import debug from 'debug'

// Base namespace for all logs
const BASE_NAMESPACE = 'rooch-sdk'

/**
 * LogLevel enum representing different logging severity levels
 */
export enum LogLevel {
  INFO = 'info',
  WARN = 'warn',
  ERROR = 'error',
  DEBUG = 'debug',
}

/**
 * Logger interface defining the standard logging methods
 */
export interface Logger {
  info(message: string, ...args: any[]): void
  warn(message: string, ...args: any[]): void
  error(message: string, ...args: any[]): void
  debug(message: string, ...args: any[]): void
}

/**
 * Format an error object for consistent logging
 */
export function formatError(error: any): string {
  if (error instanceof Error) {
    return `${error.name}: ${error.message}${error.stack ? `\n${error.stack}` : ''}`
  }
  return String(error)
}

/**
 * Standard logger implementation using debug package
 */
export class RoochLogger implements Logger {
  private infoLogger: debug.Debugger
  private warnLogger: debug.Debugger
  private errorLogger: debug.Debugger
  private debugLogger: debug.Debugger

  /**
   * Create a new logger instance for a specific module
   *
   * @param moduleName Name of the module (e.g., 'transport', 'client')
   * @param subModule Optional sub-module name (e.g., 'ws', 'http')
   */
  constructor(moduleName: string, subModule?: string) {
    const baseNamespace = subModule
      ? `${BASE_NAMESPACE}:${moduleName}:${subModule}`
      : `${BASE_NAMESPACE}:${moduleName}`

    this.infoLogger = debug(`${baseNamespace}:info`)
    this.warnLogger = debug(`${baseNamespace}:warn`)
    this.errorLogger = debug(`${baseNamespace}:error`)
    this.debugLogger = debug(`${baseNamespace}:debug`)

    // Configure colors for different levels
    this.infoLogger.color = '36' // cyan
    this.warnLogger.color = '33' // yellow
    this.errorLogger.color = '31' // red
    this.debugLogger.color = '90' // gray
  }

  /**
   * Log an informational message
   */
  info(message: string, ...args: any[]): void {
    this.infoLogger(message, ...args)
  }

  /**
   * Log a warning message
   */
  warn(message: string, ...args: any[]): void {
    this.warnLogger(message, ...args)
  }

  /**
   * Log an error message
   */
  error(message: string, ...args: any[]): void {
    this.errorLogger(message, ...args)
  }

  /**
   * Log a debug message (more verbose than info)
   */
  debug(message: string, ...args: any[]): void {
    this.debugLogger(message, ...args)
  }
}

/**
 * Create a new logger instance
 *
 * @param moduleName Name of the module (e.g., 'transport', 'client')
 * @param subModule Optional sub-module name (e.g., 'ws', 'http')
 * @returns A Logger instance
 */
export function createLogger(moduleName: string, subModule?: string): Logger {
  return new RoochLogger(moduleName, subModule)
}
