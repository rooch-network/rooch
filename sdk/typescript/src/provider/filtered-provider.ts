// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  FunctionId,
  TypeTag,
  Arg,
  AnnotatedFunctionResultView,
  AnnotatedStateView,
  ListAnnotatedStateResultPageView,
} from '../types'
import { IProvider } from './interface'

export interface ITransactionFilterChain {
  doFilter(request: any): Promise<any>
}

export interface ITransactionFilter {
  init(): void
  doFilter(request: any, chain: ITransactionFilterChain): Promise<any>
  destroy(): void
}

export class FilteredProvider implements IProvider {
  private target: IProvider
  private filters: Array<ITransactionFilter>

  public constructor(target: IProvider, filters: Array<ITransactionFilter>) {
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

  executeViewFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[] | undefined,
    args?: Arg[] | undefined,
  ): Promise<AnnotatedFunctionResultView> {
    return this.target.executeViewFunction(funcId, tyArgs, args)
  }

  getAnnotatedStates(accessPath: string): Promise<AnnotatedStateView | null[]> {
    return this.target.getAnnotatedStates(accessPath)
  }

  listAnnotatedStates(
    access_path: string,
    cursor: Uint8Array | null,
    limit: number,
  ): Promise<ListAnnotatedStateResultPageView> {
    return this.target.listAnnotatedStates(access_path, cursor, limit)
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
