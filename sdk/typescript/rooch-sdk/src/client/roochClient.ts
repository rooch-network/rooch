// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import fetch from 'isomorphic-fetch'
import { HTTPTransport, RequestManager } from '@open-rpc/client-js'
import { JsonRpcClient } from '../generated/client'
import {
  ChainInfo,
  Network,
  DevNetwork,
  DEFAULT_MAX_GAS_AMOUNT,
  RoochMultiChainID,
} from '../constants'
import {
  AnnotatedFunctionResultView,
  BalanceInfoView,
  bcs,
  EventOptions,
  EventPageView,
  InscriptionStatePageView,
  EventWithIndexerPageView,
  StatePageView,
  StateView,
  TransactionWithInfoPageView,
  TransactionWithInfoView,
  UTXOStatePageView,
  BalanceInfoPageView,
  IPage,
  ObjectStateView,
  FieldStateView,
} from '../types'
import {
  addressToListTuple,
  addressToSeqNumber,
  encodeArg,
  encodeFunctionCall,
  functionIdToStirng,
  toHexString,
  typeTagToString,
} from '../utils'
import {
  ExecuteViewFunctionParams,
  GetEventsParams,
  GetTransactionsParams,
  QueryInscriptionsParams,
  QueryUTXOsParams,
  ResoleRoochAddressParams,
  ListStatesParams,
  QueryTransactionParams,
  QueryEventParams,
  GetBalanceParams,
  GetBalancesParams,
  SessionInfoResult,
  QueryObjectStatesParams,
  QueryFieldStatesParams,
  DEFAULT_LIMIT,
  DEFAULT_NULL_CURSOR,
  DEFAULT_DISPLAY,
  GetStatesParams,
  QuerySessionKeysParams,
  SendTransactionParams,
  TransactionDataParams,
  ExecuteTransactionParams,
  ExecuteTransactionInfoParams,
} from './roochClientTypes'

import {
  AccountAddress as BCSAccountAddress,
  RoochTransaction,
  RoochTransactionData,
} from '../generated/runtime/rooch_types/mod'

import { BcsSerializer } from '../generated/runtime/bcs/bcsSerializer'
import { Buffer } from 'buffer'
import { MultiChainAddress } from '../address'

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

export class RoochClient {
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
        new HTTPTransport(network.options.url, {
          headers: {
            'Content-Type': 'application/json',
          },
          fetcher: opts.fetcher,
        }),
      ]),
    )
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

  async executeTransaction(params: ExecuteTransactionParams) {
    return await this.client.rooch_executeRawTransaction(await this.buildRawTransations(params), {
      withOutput: true,
    })
  }

  async sendRawTransaction(params: SendTransactionParams) {
    const payload = await this.buildRawTransations(params)
    return this.client.rooch_sendRawTransaction(payload)
  }

  async getTransactionsByHashes(tx_hashes: string[]): Promise<TransactionWithInfoView | null[]> {
    return await this.client.rooch_getTransactionsByHash(tx_hashes)
  }

  async getTransactions(params?: GetTransactionsParams): Promise<TransactionWithInfoPageView> {
    return this.client.rooch_getTransactionsByOrder(
      params?.cursor?.toString() || DEFAULT_NULL_CURSOR,
      params?.limit?.toString() || DEFAULT_LIMIT,
      params?.descending_order || true,
    )
  }

  // Get the events by event handle id
  async getEvents(params: GetEventsParams): Promise<EventPageView> {
    return await this.client.rooch_getEventsByEventHandle(
      params.eventHandleType,
      params?.cursor?.toString() || DEFAULT_NULL_CURSOR,
      params?.limit?.toString() || DEFAULT_LIMIT,
      params?.descending_order || true,
      { decode: true } as EventOptions,
    )
  }

  // Get the states by access_path
  async getStates(params: GetStatesParams): Promise<StateView[]> {
    const result = await this.client.rooch_getStates(
      params.accessPath,
      params.display || DEFAULT_DISPLAY,
    )

    return result as unknown as StateView[]
  }

  async listStates(params: ListStatesParams): Promise<StatePageView> {
    return await this.client.rooch_listStates(
      params.accessPath,
      params.cursor || DEFAULT_NULL_CURSOR,
      params.limit?.toString() || DEFAULT_LIMIT,
      params.display || DEFAULT_DISPLAY,
    )
  }

  async queryObjectStates(params: QueryObjectStatesParams): Promise<ObjectStateView> {
    return await this.client.rooch_queryObjectStates(
      params.filter,
      params.cursor || DEFAULT_NULL_CURSOR,
      params.limit?.toString() || DEFAULT_LIMIT,
      {
        descending: params.descending_order || true,
        showDisplay: params.showDisplay || true,
      },
    )
  }

  async queryFieldStates(params: QueryFieldStatesParams): Promise<FieldStateView> {
    return await this.client.rooch_queryFieldStates(
      params.filter,
      params.cursor || DEFAULT_NULL_CURSOR,
      params.limit?.toString() || DEFAULT_LIMIT,
      {
        descending: params.descending_order || true,
        showDisplay: params.showDisplay || true,
      },
    )
  }

  async queryInscriptions(params: QueryInscriptionsParams): Promise<InscriptionStatePageView> {
    return await this.client.btc_queryInscriptions(
      params.filter,
      params.cursor || DEFAULT_NULL_CURSOR,
      params.limit?.toString() || DEFAULT_LIMIT,
      params.descending_order || true,
    )
  }

  async queryUTXOs(params: QueryUTXOsParams): Promise<UTXOStatePageView> {
    return await this.client.btc_queryUTXOs(
      params.filter,
      params.cursor || DEFAULT_NULL_CURSOR,
      params.limit?.toString() || DEFAULT_LIMIT,
      params.descending_order || true,
    )
  }

  async queryTransactions(params: QueryTransactionParams): Promise<TransactionWithInfoPageView> {
    return await this.client.rooch_queryTransactions(
      params.filter,
      params.cursor?.toString() || DEFAULT_NULL_CURSOR,
      params.limit?.toString() || DEFAULT_LIMIT,
      {
        descending: params.descending_order || true,
        showDisplay: params.showDisplay || false,
      },
    )
  }

  async queryEvents(params: QueryEventParams): Promise<EventWithIndexerPageView> {
    return await this.client.rooch_queryEvents(
      params.filter,
      params.cursor || DEFAULT_NULL_CURSOR,
      params.limit?.toString() || DEFAULT_LIMIT,
      {
        descending: params.descending_order || true,
        showDisplay: params.showDisplay || false,
      },
    )
  }

  async getBalance(params: GetBalanceParams): Promise<BalanceInfoView> {
    return await this.client.rooch_getBalance(params.address, params.coinType)
  }

  async getBalances(params: GetBalancesParams): Promise<BalanceInfoPageView> {
    return await this.client.rooch_getBalances(
      params.address,
      params.cursor || DEFAULT_NULL_CURSOR,
      params.limit?.toString() || DEFAULT_LIMIT,
    )
  }

  /// contract func

  async getSequenceNumber(address: string): Promise<number> {
    const resp = await this.executeViewFunction({
      funcId: '0x2::account::sequence_number',
      args: [
        {
          type: 'Address',
          value: address,
        },
      ],
    })

    if (resp && resp.return_values) {
      return resp.return_values[0].decoded_value as number
    }

    return 0
  }

  /**
   * Query account's sessionKey
   *
   * @param address
   * @param cursor The page cursor
   * @param limit The page limit
   */
  public async querySessionKeys(
    params: QuerySessionKeysParams,
  ): Promise<IPage<SessionInfoResult, string>> {
    const accessPath = `/resource/${params.address}/0x3::session_key::SessionKeys`
    const states = await this.getStates({ accessPath })

    if (!states || (Array.isArray(states) && states.length === 0)) {
      throw new Error('not found state')
    }
    const stateView = states as any

    const tableId = stateView[0].decoded_value.value.keys.value.handle.value.id

    const tablePath = `/table/${tableId}`
    const pageView = await this.listStates({
      accessPath: tablePath,
      cursor: params.cursor,
      limit: params.limit,
    })

    const parseScopes = (data: Array<any>) => {
      const result = new Array<string>()

      for (const scope of data) {
        const value = scope.value
        result.push(`${value.module_address}::${value.module_name}::${value.function_name}`)
      }

      return result
    }

    const parseStateToSessionInfo = () => {
      const result = new Array<SessionInfoResult>()

      for (const state of pageView.data as any) {
        const moveValue = state?.state.decoded_value as any

        if (moveValue) {
          const val = moveValue.value

          result.push({
            appName: val.app_name,
            appUrl: val.app_url,
            authenticationKey: val.authentication_key,
            scopes: parseScopes(val.scopes),
            createTime: parseInt(val.create_time),
            lastActiveTime: parseInt(val.last_active_time),
            maxInactiveInterval: parseInt(val.max_inactive_interval),
          } as SessionInfoResult)
        }
      }
      return result
    }

    return {
      data: parseStateToSessionInfo(),
      nextCursor: pageView.next_cursor,
      hasNextPage: pageView.has_next_page,
    }
  }

  /**
   * Check session key whether expired
   *
   * @param address rooch address
   * @param authKey the auth key
   */
  async sessionIsExpired(address: string, authKey: string): Promise<boolean> {
    const result = await this.executeViewFunction({
      funcId: '0x3::session_key::is_expired_session_key',
      tyArgs: [],
      args: [
        {
          type: 'Address',
          value: address,
        },
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(authKey),
        },
      ],
    })

    if (result && result.vm_status !== 'Executed') {
      throw new Error('view 0x3::session_key::is_expired_session_key fail')
    }

    return result.return_values![0].decoded_value as boolean
  }

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
    const handleAddress = () => {
      switch (params.multiChainID) {
        case RoochMultiChainID.Bitcoin:
          return Array.from(
            new MultiChainAddress(params.multiChainID, params.address).getRawAddress(),
          )
        case RoochMultiChainID.Ether:
          return Array.from(Buffer.from(params.address.substring(2), 'hex'))
        default:
          return Array.from(Buffer.from(params.address))
      }
    }

    const ma = new bcs.MultiChainAddress(BigInt(params.multiChainID), handleAddress())

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

  private async buildRawTransations(params: ExecuteTransactionParams): Promise<Uint8Array> {
    if (params instanceof Uint8Array) {
      return params
    }

    if (
      typeof params === 'object' &&
      params !== null &&
      'authorizer' in params &&
      'data' in params
    ) {
      const { data, authorizer } = params as TransactionDataParams
      const transactionDataPayload = (() => {
        const se = new BcsSerializer()
        data.serialize(se)
        return se.getBytes()
      })()
      const auth = await authorizer.auth(transactionDataPayload)
      const transaction = new RoochTransaction(data, auth)
      return (() => {
        const se = new BcsSerializer()
        transaction.serialize(se)
        return se.getBytes()
      })()
    }

    const { address, authorizer, args, funcId, tyArgs, opts } =
      params as ExecuteTransactionInfoParams
    const number = await this.getSequenceNumber(address)
    const bcsArgs = args?.map((arg) => encodeArg(arg)) ?? []
    const scriptFunction = encodeFunctionCall(funcId, tyArgs ?? [], bcsArgs)
    const txData = new RoochTransactionData(
      new BCSAccountAddress(addressToListTuple(address)),
      BigInt(number),
      BigInt(this.getChainId()),
      BigInt(opts?.maxGasAmount ?? DEFAULT_MAX_GAS_AMOUNT),
      scriptFunction,
    )
    const transactionDataPayload = (() => {
      const se = new BcsSerializer()
      txData.serialize(se)
      return se.getBytes()
    })()
    const auth = await authorizer.auth(transactionDataPayload)
    const transaction = new RoochTransaction(txData, auth)
    return (() => {
      const se = new BcsSerializer()
      transaction.serialize(se)
      return se.getBytes()
    })()
  }
}
