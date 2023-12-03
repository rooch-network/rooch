// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { IClient } from './interface'
import {
  FilteredProvider,
  ITransactionFilter,
  ITransactionFilterChain,
  FuncFilter,
  FilterFunc,
} from './filtered-client'
import {
  FunctionId,
  TypeTag,
  Arg,
  AnnotatedFunctionResultView,
  StateView,
  StatePageView,
} from '../types'

const mockFilter: ITransactionFilter = {
  init: vi.fn(),
  doFilter: vi.fn((request, chain) => chain.doFilter(request)),
  destroy: vi.fn(),
}

const mockProvider: IClient = {
  sendRawTransaction: vi.fn((playload) => Promise.resolve('mockTransactionId')),

  getRpcApiVersion: function (): Promise<string | undefined> {
    throw new Error('Function not implemented.')
  },
  getChainId: function (): number {
    throw new Error('Function not implemented.')
  },
  executeViewFunction: function (
    funcId: FunctionId,
    tyArgs?: TypeTag[] | undefined,
    args?: Arg[] | undefined,
  ): Promise<AnnotatedFunctionResultView> {
    throw new Error('Function not implemented.')
  },
  getStates: function (accessPath: string): Promise<StateView | null[]> {
    throw new Error('Function not implemented.')
  },
  listStates: function (
    access_path: string,
    cursor: Uint8Array | null,
    limit: number,
  ): Promise<StatePageView> {
    throw new Error('Function not implemented.')
  },
}

const errorHandlingFilter: FilterFunc = async (request: any, chain: ITransactionFilterChain) => {
  try {
    return await chain.doFilter(request)
  } catch (error) {
    return 'errorHandledTransactionId'
  }
}

describe('FilteredProvider', () => {
  let filteredProvider: FilteredProvider

  beforeEach(() => {
    filteredProvider = new FilteredProvider(mockProvider, [mockFilter])
  })

  it('should call filter and provider correctly when sendRawTransaction', async () => {
    const playload = new Uint8Array()
    const result = await filteredProvider.sendRawTransaction(playload)

    expect(mockFilter.doFilter).toHaveBeenCalledWith(playload, expect.anything())
    expect(mockProvider.sendRawTransaction).toHaveBeenCalledWith(playload)
    expect(result).toBe('mockTransactionId')
  })

  it('should handle error correctly when sendRawTransaction throws error', async () => {
    mockProvider.sendRawTransaction = vi.fn(() => Promise.reject(new Error('mock error')))

    const errorHandlingProvider = new FilteredProvider(mockProvider, [
      new FuncFilter(errorHandlingFilter),
    ])

    const playload = new Uint8Array()
    const result = await errorHandlingProvider.sendRawTransaction(playload)

    expect(mockProvider.sendRawTransaction).toHaveBeenCalledWith(playload)
    expect(result).toBe('errorHandledTransactionId')
  })
})
