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
  TransactionView,
  AnnotatedFunctionResultView,
  AnnotatedStateView,
  TransactionResultPageView,
  AnnotatedEventResultPageView,
  ListAnnotatedStateResultPageView,
  StateView,
  StateResultPageView,
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
    // this.client.close()
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

  async getTransactionsByHash(tx_hashes: string[]): Promise<TransactionView | null[]> {
    return await this.client.rooch_getTransactionsByHash(tx_hashes)
  }

  // async getTransactionInfosByHash(
  //   txHashes: string[],
  // ): Promise<TransactionExecutionInfoView | null[]> {
  //   return await this.client.rooch_getTransactionInfosByHash(txHashes)
  // }

  // Get the annotated states by access_path The annotated states include the decoded move value of the state
  async getAnnotatedStates(accessPath: string): Promise<AnnotatedStateView | null[]> {
    return await this.client.rooch_getAnnotatedStates(accessPath)
  }

  async getTransactionsByOrder(cursor: number, limit: number): Promise<TransactionResultPageView> {
    return this.client.rooch_getTransactionsByOrder(cursor, limit)
  }

  // Get the events by event handle id
  async getEventsByEventHandle(
    event_handle_type: string,
    cursor: number,
    limit: number,
  ): Promise<AnnotatedEventResultPageView> {
    return await this.client.rooch_getEventsByEventHandle(event_handle_type, cursor, limit)
  }

  // Get the states by access_path
  async getStates(access_path: string): Promise<StateView | null[]> {
    return await this.client.rooch_getStates(access_path)
  }

  // List the states by access_path
  async listStates(
    access_path: string,
    cursor: Uint8Array,
    limit: number,
  ): Promise<StateResultPageView> {
    return await this.client.rooch_listStates(access_path, cursor, limit)
  }

  // TODO:
  // async getTransactionByHash(hash: string): Promise<TransactionView> {
  //   return this.client.rooch_getTransactionByHash(hash)
  // }

  //
  // // List the annotated states by access_path The annotated states include the decoded move value of the state
  // async listAnnotatedStates(
  //   access_path: string,
  //   cursor: Uint8Array,
  //   limit: number,
  // ): Promise<PageView_for_Nullable_AnnotatedStateView_and_alloc_vec_Vec_U8Array> {
  //   return await this.rpcClient.rooch_listAnnotatedStates(
  //     access_path,
  //     cursor,
  //     limit,
  //   )
  // }

  async listAnnotatedStates(
    access_path: string,
    cursor: Bytes | null,
    limit: number,
  ): Promise<ListAnnotatedStateResultPageView> {
    return await this.client.rooch_listAnnotatedStates(access_path, cursor as any, limit)
  }

  // // List the states by access_path
  // async listStates(
  //   access_path: string,
  //   cursor: Uint8Array,
  //   limit: number,
  // ): Promise<PageView_for_Nullable_StateView_and_alloc_vec_Vec_U8Array> {
  //   return await this.rpcClient.rooch_listStates(access_path, cursor, limit)
  // }
}
