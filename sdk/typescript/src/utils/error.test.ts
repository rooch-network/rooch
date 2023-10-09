// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import {
  parseRoochErrorCode,
  parseRoochErrorSubStatus,
  ErrorCategory,
  getErrorCategoryName,
} from './error'

describe('err', () => {
  describe('parseRoochErrorCode', () => {
    it('should return the correct sub status', () => {
      const errorMessage = 'status ABORTED of type Execution with sub status 66537'
      expect(parseRoochErrorCode(errorMessage)).toBe(66537)
    })

    it('should return null if no sub status is found', () => {
      const errorMessage = 'status ABORTED of type Execution with no sub status'
      expect(parseRoochErrorCode(errorMessage)).toBeNull()
    })

    it('should return null if input is not a string', () => {
      const errorMessage = null
      expect(parseRoochErrorCode(errorMessage)).toBeNull()
    })
  })

  describe('parseRoochErrorSubStatus', () => {
    it('should return the correct sub status', () => {
      const errorMessage = 'status ABORTED of type Execution with sub status 66537'
      const subStatus = parseRoochErrorSubStatus(errorMessage)
      expect(subStatus).toBeDefined()
      expect(subStatus?.category).toBe(ErrorCategory.INVALID_ARGUMENT)
      expect(subStatus?.reason).toBe(1001)
    })

    it('should return null if no sub status is found', () => {
      const errorMessage = 'status ABORTED of type Execution with no sub status'
      expect(parseRoochErrorSubStatus(errorMessage)).toBeNull()
    })

    it('should return null if input is not a string', () => {
      const errorMessage = null
      expect(parseRoochErrorSubStatus(errorMessage)).toBeNull()
    })
  })

  describe('getErrorCategoryName', () => {
    it('should return the correct string representation of the enum', () => {
      expect(getErrorCategoryName(ErrorCategory.INVALID_ARGUMENT)).toBe('INVALID_ARGUMENT')
      expect(getErrorCategoryName(ErrorCategory.OUT_OF_RANGE)).toBe('OUT_OF_RANGE')
      expect(getErrorCategoryName(ErrorCategory.INVALID_STATE)).toBe('INVALID_STATE')
      expect(getErrorCategoryName(ErrorCategory.UNAUTHENTICATED)).toBe('UNAUTHENTICATED')
      expect(getErrorCategoryName(ErrorCategory.PERMISSION_DENIED)).toBe('PERMISSION_DENIED')
      expect(getErrorCategoryName(ErrorCategory.NOT_FOUND)).toBe('NOT_FOUND')
      expect(getErrorCategoryName(ErrorCategory.ABORTED)).toBe('ABORTED')
      expect(getErrorCategoryName(ErrorCategory.ALREADY_EXISTS)).toBe('ALREADY_EXISTS')
      expect(getErrorCategoryName(ErrorCategory.RESOURCE_EXHAUSTED)).toBe('RESOURCE_EXHAUSTED')
      expect(getErrorCategoryName(ErrorCategory.CANCELLED)).toBe('CANCELLED')
      expect(getErrorCategoryName(ErrorCategory.INTERNAL)).toBe('INTERNAL')
      expect(getErrorCategoryName(ErrorCategory.NOT_IMPLEMENTED)).toBe('NOT_IMPLEMENTED')
      expect(getErrorCategoryName(ErrorCategory.UNAVAILABLE)).toBe('UNAVAILABLE')
    })
  })
})
