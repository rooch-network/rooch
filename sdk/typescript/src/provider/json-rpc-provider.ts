// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import fetch from 'isomorphic-fetch'
import { HTTPTransport, RequestManager } from '@open-rpc/client-js'
import { JsonRpcClient } from '../generated/client'
import { Chain, ChainInfo, DevChain } from '../constants/chain'
import {
  FunctionId,
  TypeTag,
  Arg,
  Bytes,
  AnnotatedFunctionResultView,
  TransactionWithInfoPageView,
  EventPageView,
  StateView,
  StatePageView,
  TransactionWithInfoView,
  StateOptions,
  EventOptions,
} from '../types'
import { functionIdToStirng, typeTagToString, encodeArg, toHexString } from '../utils'
import { IProvider } from './interface'

/**
 * Configuration options for the JsonRpcProvider. If the value of a field is not provided,
 * value in `DEFAULT_OPTIONS` for that field will be used
 */
export type RpcProviderOptions = {
  /**
   * Cache timeout in seconds for the RPC API Version
   */
  versionCacheTimeoutInSeconds?: number

  /** Allow defining a custom RPC client to use */
  fetcher?: typeof fetch
}

const DEFAULT_OPTIONS: RpcProviderOptions = {
  versionCacheTimeoutInSeconds: 600,
}

export class JsonRpcProvider implements IProvider {
  public chain: Chain

  private client: JsonRpcClient

  private rpcApiVersion: string | undefined

  private cacheExpiry: number | undefined

  constructor(chain: Chain = DevChain, public options: RpcProviderOptions = DEFAULT_OPTIONS) {
    this.chain = chain

    const opts = { ...DEFAULT_OPTIONS, ...options }
    this.options = opts

    this.client = new JsonRpcClient(
      new RequestManager([
        new HTTPTransport(chain.url, {
          headers: {
            'Content-Type': 'application/json',
          },
          fetcher: opts.fetcher,
        }),
      ]),
    )
  }

  switchChain(chain: Chain) {
    this.client.close()
    this.chain = chain
    this.client = new JsonRpcClient(
      new RequestManager([
        new HTTPTransport(chain.url, {
          headers: {
            'Content-Type': 'application/json',
          },
          fetcher: this.options.fetcher,
        }),
      ]),
    )
  }

  ChainInfo(): ChainInfo {
    return this.chain.info
  }

  getChainId(): number {
    return this.chain.id
  }

  async getRpcApiVersion(): Promise<string | undefined> {
    if (this.rpcApiVersion && this.cacheExpiry && this.cacheExpiry <= Date.now()) {
      return this.rpcApiVersion
    }

    try {
      this.rpcApiVersion = await this.client.getRpcApiVersion()
      this.cacheExpiry =
        // Date.now() is in milliseconds, but the timeout is in seconds
        Date.now() + (this.options.versionCacheTimeoutInSeconds ?? 0) * 1000
      return this.rpcApiVersion
    } catch (err) {
      return undefined
    }
  }

  // Execute a read-only function call The function do not change the state of Application
  async executeViewFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
  ): Promise<AnnotatedFunctionResultView> {
    const tyStrArgs = tyArgs?.map((v) => typeTagToString(v))
    const bcsArgs = args?.map((arg) => toHexString(encodeArg(arg))) as any

    return this.client.rooch_executeViewFunction({
      function_id: functionIdToStirng(funcId),
      ty_args: tyStrArgs ?? [],
      args: bcsArgs ?? [],
    })
  }

  // Send the signed transaction in bcs hex format
  // This method does not block waiting for the transaction to be executed.
  async sendRawTransaction(playload: Bytes): Promise<string> {
    return this.client.rooch_sendRawTransaction(playload)
  }

  async getTransactionsByHashes(tx_hashes: string[]): Promise<TransactionWithInfoView | null[]> {
    return await this.client.rooch_getTransactionsByHash(tx_hashes)
  }

  async getTransactions(
    cursor: number,
    limit: number,
  ): Promise<TransactionWithInfoPageView> {
    return this.client.rooch_getTransactionsByOrder(cursor.toString(), limit.toString())
  }

  // Get the events by event handle id
  async getEvents(
    event_handle_type: string,
    cursor: number,
    limit: number,
  ): Promise<EventPageView> {
    return await this.client.rooch_getEventsByEventHandle(
      event_handle_type,
      cursor.toString(),
      limit.toString(),
      { decode: true } as EventOptions,
    )
  }

  // Get the states by access_path
  async getState(access_path: string): Promise<StateView | null[]> {
    return await this.client.rooch_getStates(access_path, { decode: true } as StateOptions)
  }

  async getStates(
    access_path: string,
    cursor: Bytes | null,
    limit: number,
  ): Promise<StatePageView> {
    return await this.client.rooch_listStates(access_path, cursor as any, limit.toString(), {
      decode: true,
    } as StateOptions)
  }
}
