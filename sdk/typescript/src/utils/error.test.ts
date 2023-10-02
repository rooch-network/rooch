// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { parseRoochErrorCode, parseRoochErrorSubStatus, ErrorCategory } from './error'

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
})
