// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import fetch from 'isomorphic-fetch'
import { HTTPTransport, RequestManager } from '@open-rpc/client-js'
import { JsonRpcClient } from '../generated/client'
import { Connection, LocalNetConnection } from './connection'
import { bytes } from '../types/bcs'
import {
  FunctionId,
  TypeTag,
  Arg,
  AnnotatedFunctionResultView,
  TransactionView,
  AnnotatedStateView,
  TransactionExecutionInfoView,
} from '../types'
import { functionIdToStirng, typeTagToString, encodeArg, toHexString } from '../utils'

import { ROOCH_DEV_CHIAN_ID } from '../constants'

/**
 * Configuration options for the JsonRpcProvider. If the value of a field is not provided,
 * value in `DEFAULT_OPTIONS` for that field will be used
 */
export type RpcProviderOptions = {
  chainID: number

  /**
   * Cache timeout in seconds for the RPC API Version
   */
  versionCacheTimeoutInSeconds?: number

  /** Allow defining a custom RPC client to use */
  fetcher?: typeof fetch
}

const DEFAULT_OPTIONS: RpcProviderOptions = {
  chainID: ROOCH_DEV_CHIAN_ID,
  versionCacheTimeoutInSeconds: 600,
}

export class JsonRpcProvider {
  public connection: Connection

  readonly client: JsonRpcClient

  private rpcApiVersion: string | undefined

  private cacheExpiry: number | undefined

  constructor(
    connection: Connection = LocalNetConnection,
    public options: RpcProviderOptions = DEFAULT_OPTIONS,
  ) {
    this.connection = connection

    const opts = { ...DEFAULT_OPTIONS, ...options }
    this.options = opts

    this.client = new JsonRpcClient(
      new RequestManager([
        new HTTPTransport(connection.url, {
          headers: {
            'Content-Type': 'application/json',
          },
          fetcher: opts.fetcher,
        }),
      ]),
    )
  }

  getChainId(): number {
    return this.options.chainID
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
  async sendRawTransaction(playload: bytes): Promise<string> {
    return this.client.rooch_sendRawTransaction(playload)
  }

  async getTransactions(tx_hashes: string[]): Promise<TransactionView | null[]> {
    return await this.client.rooch_getTransactions(tx_hashes)
  }

  async getTransactionInfosByHash(
    txHashes: string[],
  ): Promise<TransactionExecutionInfoView | null[]> {
    return await this.client.rooch_getTransactionInfosByHash(txHashes)
  }

  // Get the annotated states by access_path The annotated states include the decoded move value of the state
  async getAnnotatedStates(accessPath: string): Promise<AnnotatedStateView | null[]> {
    return await this.client.rooch_getAnnotatedStates(accessPath)
  }

  // TODO: wait bcs

  // // Get the events by event filter
  // async getEvents(
  //   filter: EventFilterView,
  // ): Promise<AnnotatedEventView | null[]> {
  //   return await this.client.rooch_getEvents(filter)
  // }

  // // Get the events by event handle id
  // async getEventsByEventHandle(
  //   event_handle_type: string,
  //   cursor: number,
  //   limit: number,
  // ): Promise<PageView_for_Nullable_AnnotatedEventView_and_uint64> {
  //   return await this.rpcClient.rooch_getEventsByEventHandle(
  //     event_handle_type,
  //     cursor,
  //     limit,
  //   )
  // }

  // // Get the states by access_path
  // async getStates(access_path: string): Promise<StateView | null[]> {
  //   return await this.rpcClient.rooch_getStates(access_path)
  // }

  // async getTransactionByHash(hash: string): Promise<TransactionView> {
  //   return this.rpcClient.rooch_getTransactionByHash(hash)
  // }

  // async getTransactions(
  //   start: number,
  //   limit: number,
  // ): Promise<TransactionView[]> {
  //   return await this.rpcClient.rooch_getTransactions(start, limit)
  // }

  // async getTransactionInfosByHash(
  //   tx_hashes: string[],
  // ): Promise<TransactionExecutionInfoView | null[]> {
  //   return await this.rpcClient.rooch_getTransactionInfosByHash(tx_hashes)
  // }

  // async getTransactionInfosByOrder(
  //   cursor: number,
  //   limit: number,
  // ): Promise<PageView_for_Nullable_TransactionExecutionInfoView_and_uint128> {
  //   return await this.rpcClient.rooch_getTransactionInfosByOrder(
  //     cursor,
  //     limit,
  //   )
  // }

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

  // // List the states by access_path
  // async listStates(
  //   access_path: string,
  //   cursor: Uint8Array,
  //   limit: number,
  // ): Promise<PageView_for_Nullable_StateView_and_alloc_vec_Vec_U8Array> {
  //   return await this.rpcClient.rooch_listStates(access_path, cursor, limit)
  // }
}
