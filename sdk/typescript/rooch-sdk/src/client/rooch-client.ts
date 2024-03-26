// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import fetch from 'isomorphic-fetch'
import { HTTPTransport, RequestManager } from '@open-rpc/client-js'
import { JsonRpcClient } from '../generated/client'
import { ChainInfo, Network, DevNetwork } from '../constants'
import {
  AnnotatedFunctionResultView,
  BalanceInfoView,
  bcsTypes,
  Bytes,
  EventOptions,
  EventPageView,
  GlobalStateView,
  InscriptionStatePageView,
  EventWithIndexerPageView,
  StateOptions,
  StatePageView,
  StateView,
  TableStateView,
  TransactionWithInfoPageView,
  TransactionWithInfoView,
  UTXOStatePageView, BalanceInfoPageView,
} from '../types'
import {
  addressToSeqNumber,
  encodeArg,
  functionIdToStirng,
  toHexString,
  typeTagToString,
} from '../utils'
import { IClient } from './interface'
import {
  ExecuteViewFunctionParams,
  GetEventsParams,
  GetTransactionsParams,
  QueryGlobalStatesParams,
  QueryInscriptionsParams,
  QueryTableStatesParams,
  QueryUTXOsParams,
  ResoleRoochAddressParams,
  ListStatesParams,
  QueryTransactionParams,
  QueryEventParams,
  GetBalanceParams,
  GetBalancesParams,
} from './types.ts'

export const ROOCH_CLIENT_BRAND = Symbol.for('@roochnetwork/rooch-sdk')

export function isRoochClient(client: unknown): client is RoochClient {
  return (
    typeof client === 'object' &&
    client !== null &&
    (client as { [ROOCH_CLIENT_BRAND]: unknown })[ROOCH_CLIENT_BRAND] === true
  )
}

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

export class RoochClient implements IClient {
  public network: Network

  private client: JsonRpcClient

  private rpcApiVersion: string | undefined

  private cacheExpiry: number | undefined

  constructor(network: Network = DevNetwork, public options: RpcProviderOptions = DEFAULT_OPTIONS) {
    this.network = network

    const opts = { ...DEFAULT_OPTIONS, ...options }
    this.options = opts

    this.client = new JsonRpcClient(
      new RequestManager([
        new HTTPTransport(network.url, {
          headers: {
            'Content-Type': 'application/json',
          },
          fetcher: opts.fetcher,
        }),
      ]),
    )
    console.log(network)
    console.log(this.client)
  }

  switchChain(network: Network) {
    this.client.close()
    this.network = network
    this.client = new JsonRpcClient(
      new RequestManager([
        new HTTPTransport(network.url, {
          headers: {
            'Content-Type': 'application/json',
          },
          fetcher: this.options.fetcher,
        }),
      ]),
    )
  }

  ChainInfo(): ChainInfo {
    return this.network.info
  }

  getChainId(): number {
    return this.network.id
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
    params: ExecuteViewFunctionParams,
  ): Promise<AnnotatedFunctionResultView> {
    const tyStrArgs = params.tyArgs?.map((v) => typeTagToString(v))
    const bcsArgs = params.args?.map((arg) => toHexString(encodeArg(arg))) as any

    return this.client.rooch_executeViewFunction({
      function_id: functionIdToStirng(params.funcId),
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

  async getTransactions(params: GetTransactionsParams): Promise<TransactionWithInfoPageView> {
    return this.client.rooch_getTransactionsByOrder(
      params.cursor.toString(),
      params.limit.toString(),
      params.descending_order,
    )
  }

  // Get the events by event handle id
  async getEvents(params: GetEventsParams): Promise<EventPageView> {
    return await this.client.rooch_getEventsByEventHandle(
      params.eventHandleType,
      params.cursor.toString(),
      params.limit.toString(),
      params.descending_order,
      { decode: true } as EventOptions,
    )
  }

  // Get the states by access_path
  async getStates(access_path: string): Promise<StateView | null[]> {
    return await this.client.rooch_getStates(access_path, { decode: true } as StateOptions)
  }

  // TODO: bug? next_cursor The true type is string
  async listStates(params: ListStatesParams): Promise<StatePageView> {
    return await this.client.rooch_listStates(
      params.accessPath,
      params.cursor as any,
      params.limit.toString(),
      {
        decode: true,
      } as StateOptions,
    )
  }

  async queryGlobalStates(params: QueryGlobalStatesParams): Promise<GlobalStateView> {
    return await this.client.rooch_queryGlobalStates(
      params.filter,
      params.cursor as any,
      params.limit.toString(),
      params.descending_order,
    )
  }

  async queryTableStates(params: QueryTableStatesParams): Promise<TableStateView> {
    return await this.client.rooch_queryTableStates(
      params.filter,
      params.cursor as any,
      params.limit.toString(),
      params.descending_order,
    )
  }

  async queryInscriptions(params: QueryInscriptionsParams): Promise<InscriptionStatePageView> {
    return await this.client.btc_queryInscriptions(
      params.filter as any,
      params.cursor as any,
      params.limit.toString(),
      params.descending_order,
    )
  }

  async queryUTXOs(params: QueryUTXOsParams): Promise<UTXOStatePageView> {
    return await this.client.btc_queryUTXOs(
      params.filter as any,
      params.cursor as any,
      params.limit.toString(),
      params.descending_order,
    )
  }

  async queryTransactions(params: QueryTransactionParams): Promise<TransactionWithInfoPageView> {
    return await this.client.rooch_queryTransactions(
      params.filter,
      params.cursor,
      params.limit,
      params.descending_order,
    )
  }

  async queryEvents(params: QueryEventParams): Promise<EventWithIndexerPageView> {
    return await this.client.rooch_queryEvents(
      params.filter,
      params.cursor,
      params.limit,
      params.descending_order,
    )
  }

  async getBalance(params: GetBalanceParams): Promise<BalanceInfoView> {
    return await this.client.rooch_getBalance(params.address, params.coinType)
  }

  async getBalances(params: GetBalancesParams): Promise<BalanceInfoPageView> {
    return await this.client.rooch_getBalances(params.address, params.cursor, params.limit)
  }

  /// contract func

  async gasCoinBalance(address: string): Promise<bigint> {
    const result = await this.executeViewFunction({
      funcId: '0x3::gas_coin::balance',
      tyArgs: [],
      args: [
        {
          type: 'Address',
          value: address,
        },
      ],
    })

    if (result && result.vm_status !== 'Executed') {
      throw new Error('view 0x3::gas_coin::balance fail')
    }

    return BigInt(result.return_values![0].decoded_value as string)
  }

  // Resolve the rooch address
  async resoleRoochAddress(params: ResoleRoochAddressParams): Promise<string> {
    const ma = new bcsTypes.MultiChainAddress(
      BigInt(params.multiChainID),
      addressToSeqNumber(params.address),
    )

    const result = await this.executeViewFunction({
      funcId: '0x3::address_mapping::resolve_or_generate',
      tyArgs: [],
      args: [
        {
          type: {
            Struct: {
              address: '0x3',
              module: 'address_mapping',
              name: 'MultiChainAddress',
            },
          },
          value: ma,
        },
      ],
    })

    if (result && result.vm_status === 'Executed' && result.return_values) {
      return result.return_values[0].decoded_value as string
    }

    throw new Error('resolve rooch address fail')
  }
}
