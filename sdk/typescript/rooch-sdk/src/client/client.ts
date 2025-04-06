// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Args } from '../bcs/index.js'
import { Signer } from '../crypto/index.js'
import { CreateSessionArgs, Session } from '../session/index.js'
import {
  decodeToRoochAddressStr,
  decodeToPackageAddressStr,
  BitcoinAddress,
  BitcoinNetowkType,
  RoochAddress,
} from '../address/index.js'
import { address, Bytes, u64 } from '../types/index.js'
import { fromHEX, str } from '../utils/index.js'
import { RoochTransport } from './transportInterface.js'
import { RoochHTTPTransport } from './httpTransport.js'
import {
  CallFunction,
  CallFunctionArgs,
  TypeArgs,
  Transaction,
  normalizeTypeArgsToStr,
} from '../transactions/index.js'
import {
  AnnotatedFunctionResultView,
  BalanceInfoView,
  ExecuteTransactionResponseView,
  GetBalanceParams,
  GetStatesParams,
  ListStatesParams,
  PaginatedStateKVViews,
  PaginationArguments,
  PaginationResult,
  SessionInfoView,
  ObjectStateView,
  QueryUTXOsParams,
  PaginatedUTXOStateViews,
  PaginatedInscriptionStateViews,
  QueryInscriptionsParams,
  GetBalancesParams,
  PaginatedBalanceInfoViews,
  QueryObjectStatesParams,
  PaginatedIndexerObjectStateViews,
  QueryTransactionsParams,
  PaginatedTransactionWithInfoViews,
  PaginatedEventViews,
  GetEventsByEventHandleParams,
  QueryEventsParams,
  PaginatedIndexerEventViews,
  ModuleABIView,
  GetModuleABIParams,
  BroadcastTXParams,
  GetObjectStatesParams,
  GetFieldStatesParams,
  ListFieldStatesParams,
  GetTransactionsByHashParams,
  TransactionWithInfoView,
  GetTransactionsByOrderParams,
  RepairIndexerParams,
  SyncStatesParams,
  PaginatedStateChangeSetWithTxOrderViews,
  DryRunRawTransactionParams,
  DryRunTransactionResponseView,
  EventFilterView,
  TransactionFilterView,
  IndexerEventView,
} from './types/index.js'
import { fixedBalance } from '../utils/balance.js'
import { RoochSubscriptionTransport, Subscription } from './subscriptionTransportInterface.js'

const DEFAULT_GAS = 50000000

/**
 * Configuration options for the RoochClient
 * You must provide either a `url` or a `transport`
 */
export type RoochClientOptions = NetworkOrTransport

type NetworkOrTransport =
  | {
      url: string
      transport?: never
      subscriptionTransport?: never
    }
  | {
      transport: RoochTransport
      subscriptionTransport?: RoochSubscriptionTransport
      url?: never
    }

const ROOCH_CLIENT_BRAND = Symbol.for('@roochnetwork/RoochClient')

export function isRoochClient(client: unknown): client is RoochClient {
  return (
    typeof client === 'object' &&
    client !== null &&
    (client as { [ROOCH_CLIENT_BRAND]: unknown })[ROOCH_CLIENT_BRAND] === true
  )
}

type SubscriptionEvent =
  | { type: 'event'; data: IndexerEventView }
  | { type: 'transaction'; data: TransactionWithInfoView }

export interface SubscriptionOptions {
  type: 'event' | 'transaction' // Subscription type
  filter?: EventFilterView | TransactionFilterView // Optional Rust-defined filter
  onEvent: (event: SubscriptionEvent) => void // Callback for received events
  onError?: (error: Error) => void // Optional callback for errors
}

export class RoochClient {
  protected chainID: bigint | undefined
  protected transport: RoochTransport
  protected subscriptionTransport?: RoochSubscriptionTransport
  private subscriptions: Map<string, SubscriptionOptions> = new Map()

  get [ROOCH_CLIENT_BRAND]() {
    return true
  }

  getTransport() {
    return this.transport
  }

  getSubscriptionTransport() {
    return this.subscriptionTransport
  }

  /**
   * Establish a connection to a rooch RPC endpoint
   *
   * @param options configuration options for the API Client
   */
  constructor(options: RoochClientOptions) {
    this.transport = options.transport ?? new RoochHTTPTransport({ url: options.url })

    this.subscriptionTransport = options.subscriptionTransport
    if (this.subscriptionTransport) {
      // Register subscription event listeners
      this.subscriptionTransport.onMessage((msg) => this.handleSubscribeMessage(msg))
      // Register reconnection listener for re-subscription
      this.subscriptionTransport.onReconnected(() => this.resubscribeAll())
      // Register error listener for transport-level errors
      this.subscriptionTransport.onError((error) => this.handleError(error))
    }
  }

  async getRpcApiVersion(): Promise<string | undefined> {
    const resp = await this.transport.request<{ info: { version: string } }>({
      method: 'rpc.discover',
      params: [],
    })

    return resp.info.version
  }

  async getChainId(): Promise<u64> {
    if (this.chainID) {
      return this.chainID
    }

    return this.transport.request({
      method: 'rooch_getChainID',
      params: [],
    })
  }

  async executeViewFunction(input: CallFunctionArgs): Promise<AnnotatedFunctionResultView> {
    const callFunction = new CallFunction(input)

    return await this.transport.request({
      method: 'rooch_executeViewFunction',
      params: [
        {
          function_id: callFunction.functionId(),
          args: callFunction.encodeArgs(),
          ty_args: callFunction.typeArgs,
        },
      ],
    })
  }

  async dryrun(input: DryRunRawTransactionParams): Promise<DryRunTransactionResponseView> {
    return await this.transport.request({
      method: 'rooch_dryRunRawTransaction',
      params: [input.txBcsHex],
    })
  }

  async signAndExecuteTransaction({
    transaction,
    signer,
    option = { withOutput: true },
  }: {
    transaction: Transaction | Bytes
    signer: Signer
    option?: {
      withOutput: boolean
    }
  }): Promise<ExecuteTransactionResponseView> {
    let transactionHex: string

    if (transaction instanceof Uint8Array) {
      transactionHex = str('hex', transaction)
    } else {
      let sender = signer.getRoochAddress().toHexAddress()
      transaction.setChainId(await this.getChainId())
      transaction.setSeqNumber(await this.getSequenceNumber(sender))
      transaction.setSender(sender)

      // need dry_run
      if (!transaction.getMaxGas()) {
        transaction.setMaxGas(DEFAULT_GAS)
        // const s = transaction.encodeData().toHex()
        // const result = await this.dryrun({ txBcsHex: s })
        //
        // if (result.raw_output.status.type === 'executed') {
        //   transaction.setMaxGas(Math.ceil(Number(result.raw_output.gas_used) * 100))
        // } else {
        //   // TODO: abort?
        //   throw Error(result.raw_output.status.type)
        // }
      }

      const auth = await signer.signTransaction(transaction)

      transaction.setAuth(auth)

      transactionHex = `0x${transaction.encode().toHex()}`
    }

    return await this.transport.request({
      method: 'rooch_executeRawTransaction',
      params: [transactionHex, option],
    })
  }

  async repairIndexer(input: RepairIndexerParams) {
    await this.transport.request({
      method: 'rooch_repairIndexer',
      params: [input.repairType, input.repairParams],
    })
  }

  async syncStates(input: SyncStatesParams): Promise<PaginatedStateChangeSetWithTxOrderViews> {
    const opt = input.queryOption || {
      decode: true,
      showDisplay: true,
    }
    return await this.transport.request({
      method: 'rooch_syncStates',
      params: [input.filter, input.cursor, input.limit, opt],
    })
  }

  // Get the states by access_path
  async getStates(input: GetStatesParams): Promise<ObjectStateView[]> {
    const opt = input.stateOption || {
      decode: true,
      showDisplay: true,
    }
    const result = await this.transport.request({
      method: 'rooch_getStates',
      params: [input.accessPath, opt],
    })

    const typedResult = result as unknown as ObjectStateView[]
    return typedResult[0] === null ? [] : typedResult
  }

  async listStates(input: ListStatesParams): Promise<PaginatedStateKVViews> {
    const opt = input.stateOption || {
      decode: true,
      showDisplay: true,
    }
    return await this.transport.request({
      method: 'rooch_listStates',
      params: [input.accessPath, input.cursor, input.limit, opt],
    })
  }

  async getModuleAbi(input: GetModuleABIParams): Promise<ModuleABIView> {
    return await this.transport.request({
      method: 'rooch_getModuleABI',
      params: [input.moduleAddr, input.moduleName],
    })
  }

  async getEvents(input: GetEventsByEventHandleParams): Promise<PaginatedEventViews> {
    const opt = input.eventOptions || {
      decode: true,
    }
    return await this.transport.request({
      method: 'rooch_getEventsByEventHandle',
      params: [input.eventHandle, input.cursor, input.limit, input.descendingOrder, opt],
    })
  }

  async queryEvents(input: QueryEventsParams): Promise<PaginatedIndexerEventViews> {
    if (typeof input.filter === 'object' && 'sender' in input.filter) {
      if (input.filter.sender === '') {
        throw Error('Invalid Address')
      }
    }

    if (typeof input.filter === 'object' && 'event_type_with_sender' in input.filter) {
      if (input.filter.event_type_with_sender.sender === '') {
        throw Error('Invalid Address')
      }
    }

    const opt = input.queryOption || {
      decode: true,
      showDisplay: true,
    }
    return await this.transport.request({
      method: 'rooch_queryEvents',
      params: [input.filter, input.cursor, input.limit, opt],
    })
  }

  async queryInscriptions(input: QueryInscriptionsParams): Promise<PaginatedInscriptionStateViews> {
    if (typeof input.filter !== 'string' && 'owner' in input.filter) {
      if (input.filter.owner === '') {
        throw Error('Invalid Address')
      }
    }
    return await this.transport.request({
      method: 'btc_queryInscriptions',
      params: [input.filter, input.cursor, input.limit, input.descendingOrder],
    })
  }

  async queryUTXO(input: QueryUTXOsParams): Promise<PaginatedUTXOStateViews> {
    if (typeof input.filter !== 'string' && 'owner' in input.filter) {
      if (input.filter.owner === '') {
        throw Error('Invalid Address')
      }
    }
    return this.transport.request({
      method: 'btc_queryUTXOs',
      params: [input.filter, input.cursor, input.limit, input.descendingOrder],
    })
  }

  async broadcastBitcoinTX(input: BroadcastTXParams): Promise<string> {
    return this.transport.request({
      method: 'btc_broadcastTX',
      params: [input.hex, input.maxfeerate, input.maxburnamount],
    })
  }

  async getObjectStates(input: GetObjectStatesParams): Promise<ObjectStateView[]> {
    const idsStr = input.ids.join(',')
    const opt = input.stateOption || {
      decode: true,
      showDisplay: true,
    }
    return this.transport.request({
      method: 'rooch_getObjectStates',
      params: [idsStr, opt],
    })
  }

  async getFieldStates(input: GetFieldStatesParams): Promise<ObjectStateView[]> {
    const opt = input.stateOption || {
      decode: true,
      showDisplay: true,
    }

    return this.transport.request({
      method: 'rooch_getFieldStates',
      params: [input.objectId, input.fieldKey, opt],
    })
  }

  async listFieldStates(input: ListFieldStatesParams): Promise<PaginatedStateKVViews> {
    const opt = input.stateOption || {
      decode: true,
      showDisplay: true,
    }

    return this.transport.request({
      method: 'rooch_listFieldStates',
      params: [input.objectId, input.cursor, input.limit, opt],
    })
  }

  async queryObjectStates(
    input: QueryObjectStatesParams,
  ): Promise<PaginatedIndexerObjectStateViews> {
    if ('owner' in input.filter) {
      if (input.filter.owner === '') {
        throw Error('Invalid Address')
      }
    }

    if ('object_type_with_owner' in input.filter) {
      if (input.filter.object_type_with_owner.owner === '') {
        throw Error('Invalid Address')
      }
    }

    const opt = input.queryOption || {
      decode: true,
      showDisplay: true,
    }
    return this.transport.request({
      method: 'rooch_queryObjectStates',
      params: [input.filter, input.cursor, input.limit, opt],
    })
  }

  async getTransactionsByHash(
    input: GetTransactionsByHashParams,
  ): Promise<TransactionWithInfoView> {
    return this.transport.request({
      method: 'rooch_getTransactionsByHash',
      params: [input.txHashes],
    })
  }

  async getTransactionsByOrder(
    input: GetTransactionsByOrderParams,
  ): Promise<PaginatedTransactionWithInfoViews> {
    return this.transport.request({
      method: 'rooch_queryTransactions',
      params: [input.cursor, input.limit, input.descendingOrder],
    })
  }

  async queryTransactions(
    input: QueryTransactionsParams,
  ): Promise<PaginatedTransactionWithInfoViews> {
    if (typeof input.filter === 'object' && 'sender' in input.filter) {
      if (input.filter.sender === '') {
        throw Error('Invalid Address')
      }
    }
    const opt = input.queryOption || {
      decode: true,
      showDisplay: true,
    }
    return this.transport.request({
      method: 'rooch_queryTransactions',
      params: [input.filter, input.cursor, input.limit, opt],
    })
  }

  // helper fn

  async getSequenceNumber(address: string): Promise<u64> {
    const resp = await this.executeViewFunction({
      target: '0x2::account::sequence_number',
      args: [Args.address(address)],
    })

    if (resp && resp.return_values) {
      return BigInt(resp.return_values?.[0]?.decoded_value as number)
    }

    return BigInt(0)
  }

  /**
   * Get the total coin balance for one coin type, owned by the address owner.
   */
  async getBalance(input: GetBalanceParams): Promise<BalanceInfoView> {
    const owner = decodeToRoochAddressStr(input.owner)

    let balanceInfoView: BalanceInfoView = await this.transport.request({
      method: 'rooch_getBalance',
      params: [owner, input.coinType],
    })

    balanceInfoView.fixedBalance = fixedBalance(balanceInfoView.balance, balanceInfoView.decimals)

    return balanceInfoView
  }

  async getBalances(input: GetBalancesParams): Promise<PaginatedBalanceInfoViews> {
    const owner = decodeToRoochAddressStr(input.owner)

    // balanceInfoView.fixedBalance = fixedBalance(balanceInfoView.balance, balanceInfoView.decimals)
    const result: PaginatedBalanceInfoViews = await this.transport.request({
      method: 'rooch_getBalances',
      params: [owner, input.cursor, input.limit],
    })

    result.data.forEach((item) => {
      item.fixedBalance = fixedBalance(item.balance, item.decimals)
    })

    return result
  }

  async transfer(input: {
    signer: Signer
    recipient: address
    amount: number | bigint
    coinType: TypeArgs
  }) {
    const recipient = decodeToRoochAddressStr(input.recipient)
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::transfer::transfer_coin',
      args: [Args.address(recipient), Args.u256(BigInt(input.amount))],
      typeArgs: [normalizeTypeArgsToStr(input.coinType)],
    })

    return await this.signAndExecuteTransaction({
      transaction: tx,
      signer: input.signer,
    })
  }

  async transferObject(input: {
    signer: Signer
    recipient: address
    objectId: string
    objectType: TypeArgs
  }) {
    const recipient = decodeToRoochAddressStr(input.recipient)
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::transfer::transfer_object',
      args: [Args.address(recipient), Args.objectId(input.objectId)],
      typeArgs: [normalizeTypeArgsToStr(input.objectType)],
    })

    return await this.signAndExecuteTransaction({
      transaction: tx,
      signer: input.signer,
    })
  }

  async resolveBTCAddress(input: {
    roochAddress: string | RoochAddress
    network: BitcoinNetowkType
  }): Promise<BitcoinAddress | undefined> {
    const address = decodeToRoochAddressStr(input.roochAddress)
    const result = await this.executeViewFunction({
      target: '0x3::address_mapping::resolve_bitcoin',
      args: [Args.address(address)],
    })

    if (result.vm_status === 'Executed' && result.return_values) {
      const value = (result.return_values?.[0]?.decoded_value as { value: any }).value

      const address =
        value && value.vec
          ? //compatible with old option version
            (((value as any).vec as any).value[0] as Array<string>)[0]
          : ((value as any).bytes as string)

      return new BitcoinAddress(address, input.network)
    }

    return undefined
  }

  async createSession({ sessionArgs, signer }: { sessionArgs: CreateSessionArgs; signer: Signer }) {
    return Session.CREATE({
      ...sessionArgs,
      client: this,
      signer: signer,
    })
  }

  async removeSession({ authKey, signer }: { authKey: string; signer: Signer }): Promise<boolean> {
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::session_key::remove_session_key_entry',
      args: [Args.vec('u8', Array.from(fromHEX(authKey)))],
    })

    return (
      (
        await this.signAndExecuteTransaction({
          transaction: tx,
          signer,
        })
      ).execution_info.status.type === 'executed'
    )
  }

  async sessionIsExpired({
    address,
    authKey,
  }: {
    address: address
    authKey: string
  }): Promise<boolean> {
    const _address = decodeToRoochAddressStr(address)
    const result = await this.executeViewFunction({
      target: '0x3::session_key::is_expired_session_key',
      args: [Args.address(_address), Args.vec('u8', Array.from(fromHEX(authKey)))],
    })

    if (result.vm_status !== 'Executed') {
      throw new Error('view 0x3::session_key::is_expired_session_key fail')
    }

    return result.return_values![0]?.decoded_value as boolean
  }

  async getAllModules({
    package_address,
    limit,
    cursor,
  }: {
    package_address: address
  } & PaginationArguments<string>): Promise<Map<string, string>> {
    const packageObjectID = `0x14481947570f6c2f50d190f9a13bf549ab2f0c9debc41296cd4d506002379659${decodeToPackageAddressStr(package_address)}`
    const result = await this.transport.request({
      method: 'rooch_listFieldStates',
      params: [packageObjectID, cursor, limit, { decode: true }],
    })

    const moduleInfo = result as unknown as ObjectStateView[]
    const moduleMap = new Map<string, string>()

    if (moduleInfo && typeof moduleInfo === 'object' && 'data' in moduleInfo) {
      const { data } = moduleInfo
      if (Array.isArray(data)) {
        for (const item of data) {
          const decodedValue = item?.state?.decoded_value

          if (decodedValue) {
            const name = decodedValue?.value?.name
            const byte_codes = decodedValue?.value?.value?.value?.byte_codes
            if (name && byte_codes) {
              moduleMap.set(name, byte_codes)
            }
          }
        }
      }
    }

    return moduleMap
  }

  async getSessionKeys({
    address,
    limit,
    cursor,
  }: {
    address: address
  } & PaginationArguments<string>): Promise<PaginationResult<string, SessionInfoView>> {
    const _address = decodeToRoochAddressStr(address)
    const accessPath = `/resource/${_address}/0x3::session_key::SessionKeys`
    const states = await this.getStates({
      accessPath,
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })

    if (states.length === 0) {
      return {
        data: [],
        hasNextPage: false,
      }
    }
    // Maybe we should define the type?
    const tableId = (
      (((states?.[0]?.decoded_value as any).value['value'] as any).value['keys'] as any).value[
        'handle'
      ] as any
    ).value['id'] as string

    const tablePath = `/table/${tableId}`

    const statePage = await this.listStates({
      accessPath: tablePath,
      cursor,
      limit: limit?.toString(),
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })

    const parseScopes = (data: Array<any>) => {
      const result = new Array<string>()

      for (const scope of data) {
        const [pkg, mod, fn] = [scope[0], scope[1], scope[2]]
        result.push(`${pkg}::${mod}::${fn}`)
      }

      return result
    }

    const parseStateToSessionInfo = () => {
      const result = new Array<SessionInfoView>()

      for (const state of statePage.data as any) {
        const moveValue = state?.state?.decoded_value as any

        if (moveValue) {
          const val = moveValue.value.value.value

          result.push({
            appName: val.app_name,
            appUrl: val.app_url,
            authenticationKey: val.authentication_key,
            scopes: parseScopes(val.scopes.value),
            createTime: parseInt(val.create_time),
            lastActiveTime: parseInt(val.last_active_time),
            maxInactiveInterval: parseInt(val.max_inactive_interval),
          } as SessionInfoView)
        }
      }
      return result.sort((a, b) => b.createTime - a.createTime)
    }

    return {
      data: parseStateToSessionInfo(),
      cursor: statePage.next_cursor,
      hasNextPage: statePage.has_next_page,
    }
  }

  /**
   * Subscribe to events or transactions
   * @param options Subscription options including type, filter, onEvent, onError
   * @returns A Subscription object containing the subscription ID and unsubscribe method
   */
  async subscribe(options: SubscriptionOptions): Promise<Subscription> {
    if (!this.subscriptionTransport) {
      throw new Error('Subscription transport is not configured')
    }

    const { type, filter } = options
    const method = type === 'event' ? 'rooch_subscribeEvents' : 'rooch_subscribeTransactions'
    const params = filter ? [filter as any] : ['all']
    const request = {
      method,
      params,
    }

    const subscription = await this.subscriptionTransport.subscribe(request)
    this.subscriptions.set(subscription.id, options)
    return subscription
  }

  private handleSubscribeMessage(msg: any): void {
    const subscriptionId = msg.params.subscription
    const options = this.subscriptions.get(subscriptionId)
    if (options) {
      if (msg.method === 'rooch_subscribeEvents') {
        options.onEvent({
          type: 'event',
          data: msg.params.result,
        })
      } else if (msg.method === 'rooch_subscribeTransactions') {
        options.onEvent({
          type: 'transaction',
          data: msg.params.result,
        })
      }
    }
  }

  private resubscribeAll(): void {
    console.log('Re-subscribing to all subscriptions...')

    if (!this.subscriptionTransport) {
      return
    }

    for (const [_id, options] of this.subscriptions) {
      console.log(`Re-subscribing to ${options.type} with ID: ${_id}`)
      this.subscribe(options)
    }
  }

  private handleError(error: Error): void {
    console.error('Transport error:', error.message)
    // Custom logic: Notify users, fallback to polling, etc.
  }

  /**
   * Unsubscribe from a specific subscription
   * @param subscriptionId The subscription ID
   */
  unsubscribe(subscriptionId: string): void {
    if (!this.subscriptionTransport) {
      throw new Error('Subscription transport is not configured')
    }

    this.subscriptionTransport.unsubscribe(subscriptionId)
  }

  destroy(): void {
    this.transport.destroy()
    this.subscriptions.clear()
  }
}
