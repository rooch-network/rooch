// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { AnnotatedFunctionResultView, StatePageView, StateView } from '../types'
import { IClient } from './interface'

import { ExecuteViewFunctionParams, ListStatesParams } from './roochClientTypes'

export interface ITransactionFilterChain {
  doFilter(request: any): Promise<any>
}

export interface ITransactionFilter {
  init(): void
  doFilter(request: any, chain: ITransactionFilterChain): Promise<any>
  destroy(): void
}

export type FilterFunc = (request: any, chain: ITransactionFilterChain) => Promise<any>

export class FuncFilter implements ITransactionFilter {
  private func: FilterFunc

  public constructor(func: FilterFunc) {
    this.func = func
  }

  init() {}
  destroy(): void {}

  async doFilter(request: any, chain: ITransactionFilterChain): Promise<any> {
    return await this.func(request, chain)
  }
}

export class FilteredProvider implements IClient {
  private target: IClient
  private filters: Array<ITransactionFilter>

  public constructor(target: IClient, filters: Array<ITransactionFilter>) {
    this.target = target
    this.filters = filters

    for (const filter of this.filters) {
      filter.init()
    }
  }

  getRpcApiVersion(): Promise<string | undefined> {
    return this.target.getRpcApiVersion()
  }

  getChainId(): number {
    return this.target.getChainId()
  }

  executeViewFunction(params: ExecuteViewFunctionParams): Promise<AnnotatedFunctionResultView> {
    return this.target.executeViewFunction(params)
  }

  getStates(accessPath: string): Promise<StateView | null[]> {
    return this.target.getStates(accessPath)
  }

  listStates(params: ListStatesParams): Promise<StatePageView> {
    return this.target.listStates(params)
  }

  sendRawTransaction(playload: Uint8Array): Promise<string> {
    let index = 0

    const chain: ITransactionFilterChain = {
      doFilter: async (req: any) => {
        if (index < this.filters.length) {
          const filter = this.filters[index++]
          return await filter.doFilter(req, chain)
        } else {
          return await this.target.sendRawTransaction(req)
        }
      },
    }

    return new Promise(async (resolve, reject) => {
      try {
        let response = await chain.doFilter(playload)
        resolve(response)
      } catch (error) {
        reject(error)
      }
    })
  }

  public destroy() {
    for (const filter of this.filters) {
      filter.destroy()
    }
  }
}
