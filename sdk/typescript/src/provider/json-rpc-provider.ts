// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import fetch from "isomorphic-fetch";
import { HTTPTransport, RequestManager } from '@open-rpc/client-js'
import { JsonRpcClient } from '../generated/client'
import { Connection, LocalnetConnection } from './connection'
import { encodeFunctionCall } from '../utils'
import { BcsSerializer, bytes } from '../types/bcs'
import { FunctionId, TypeTag, functionIdToStirng } from '../types'
import {
  // AnnotatedEventView,
  AnnotatedFunctionReturnValueView,
  // AnnotatedStateView,
  // EventFilterView,
  // FunctionCallView,
  // PageView_for_Nullable_AnnotatedEventView_and_uint64,
  // PageView_for_Nullable_AnnotatedStateView_and_alloc_vec_Vec_U8Array,
  // PageView_for_Nullable_StateView_and_alloc_vec_Vec_U8Array,
  // PageView_for_Nullable_TransactionExecutionInfoView_and_uint128,
  // StateView,
  // TransactionExecutionInfoView,
  // TransactionView,
} from '../generated/client/types'

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
  fetcher?: typeof fetch;
}

const DEFAULT_OPTIONS: RpcProviderOptions = {
  versionCacheTimeoutInSeconds: 600,
}

export class JsonRpcProvider {
  public connection: Connection

  readonly client: JsonRpcClient

  private rpcApiVersion: string | undefined

  private cacheExpiry: number | undefined

  constructor(
    connection: Connection = LocalnetConnection,
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

    this.client.onError((e)=>{
      console.error("client error", e)
    })

    this.client.onNotification((data)=>{
      console.log("data:", data)
    })
  }

  async getRpcApiVersion(): Promise<string | undefined> {
    if (
      this.rpcApiVersion &&
      this.cacheExpiry &&
      this.cacheExpiry <= Date.now()
    ) {
      return this.rpcApiVersion
    }

    try {
      this.client.getRpcApiVersion()
      const resp = await this.client.getRpcApiVersion()
      this.rpcApiVersion = resp
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
    args?: Uint8Array[],
    tyArgs?: string[],
  ): Promise<AnnotatedFunctionReturnValueView[]> {
    // let _args = args.map((v) => {
    //   let se = new BcsSerializer()
    //   typeTagToSCS(v).serialize(se)
    //   return se.getBytes()
    // })

    // rooch, eth, wellet,
    // TDOO: args, tyArgs, wait bcs
    return await this.client.rooch_executeViewFunction({
      function_id: functionIdToStirng(funcId),
      args: args ?? [],
      ty_args: tyArgs ?? [],
    })
  }

  // Send the signed transaction in bcs hex format
  // This method does not block waiting for the transaction to be executed.
  async signAndExecuteFunction(
    functionId: FunctionId,
    tyArgs: TypeTag[],
    args: bytes[],
  ): Promise<string> {
    // TODO: The bcs type is faulty

    const ser = new BcsSerializer()
    encodeFunctionCall(functionId, tyArgs, args).serialize(ser)

    return this.client.rooch_sendRawTransaction(ser.getBytes())
  }

  // TODO: wait bcs
  // // Get the annotated states by access_path The annotated states include the decoded move value of the state
  // async getAnnotatedStates(
  //   access_path: string,
  // ): Promise<AnnotatedStateView | null[]> {
  //   return await this.rpcClient.rooch_getAnnotatedStates(access_path)
  // }

  // // Get the events by event filter
  // async getEvents(
  //   filter: EventFilterView,
  // ): Promise<AnnotatedEventView | null[]> {
  //   return await this.rpcClient.rooch_getEvents(filter)
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

  // async getTransactionByIndex(
  //   start: number,
  //   limit: number,
  // ): Promise<TransactionView[]> {
  //   return await this.rpcClient.rooch_getTransactionByIndex(start, limit)
  // }

  // async getTransactionInfosByTxHash(
  //   tx_hashes: string[],
  // ): Promise<TransactionExecutionInfoView | null[]> {
  //   return await this.rpcClient.rooch_getTransactionInfosByTxHash(tx_hashes)
  // }

  // async getTransactionInfosByTxOrder(
  //   cursor: number,
  //   limit: number,
  // ): Promise<PageView_for_Nullable_TransactionExecutionInfoView_and_uint128> {
  //   return await this.rpcClient.rooch_getTransactionInfosByTxOrder(
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

