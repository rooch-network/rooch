// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Args } from '../bcs/index.js'
import { Signer } from '../crypto/index.js'
import { CreateSessionArgs, Session } from '../session/index.js'
import {
  decodeToRoochAddressStr,
  BitcoinAddress,
  BitcoinNetowkType,
  RoochAddress,
} from '../address/index.js'
import { address, Bytes, u64 } from '../types/index.js'
import { fromHEX, str } from '../utils/index.js'
import { RoochHTTPTransport, RoochTransport } from './httpTransport.js'
import {
  CallFunction,
  CallFunctionArgs,
  TypeArgs,
  Transaction,
  normalizeTypeArgsToStr,
} from '../transactions/index.js'
import {
  AnnotatedFunctionResultView,
  AnnotatedMoveStructView,
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
} from './types/index.js'
import { fixedBalance } from '../utils/balance.js'

/**
 * Configuration options for the RoochClient
 * You must provide either a `url` or a `transport`
 */
export type RoochClientOptions = NetworkOrTransport

type NetworkOrTransport =
  | {
      url: string
      transport?: never
    }
  | {
      transport: RoochTransport
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

export class RoochClient {
  protected chainID: bigint | undefined
  protected transport: RoochTransport

  get [ROOCH_CLIENT_BRAND]() {
    return true
  }

  /**
   * Establish a connection to a rooch RPC endpoint
   *
   * @param options configuration options for the API Client
   */
  constructor(options: RoochClientOptions) {
    this.transport = options.transport ?? new RoochHTTPTransport({ url: options.url })
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

      const auth = await signer.signTransaction(transaction)

      transaction.setAuth(auth)

      transactionHex = `0x${transaction.encode().toHex()}`
    }

    return await this.transport.request({
      method: 'rooch_executeRawTransaction',
      params: [transactionHex, option],
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
    const opt = input.eventHandleType || {
      decode: true,
    }
    return await this.transport.request({
      method: 'rooch_getEventsByEventHandle',
      params: [input.eventHandleType, input.cursor, input.limit, input.descendingOrder, opt],
    })
  }

  async queryEvents(input: QueryEventsParams): Promise<PaginatedIndexerEventViews> {
    if ('sender' in input.filter) {
      if (input.filter.sender === '') {
        throw Error('Invalid Address')
      }
    }

    if ('event_type_with_sender' in input.filter) {
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

  // Query the Inscription via global index by Inscription filter
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

  async queryTransactions(
    input: QueryTransactionsParams,
  ): Promise<PaginatedTransactionWithInfoViews> {
    if ('sender' in input.filter) {
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
      const value = (result.return_values?.[0]?.decoded_value as AnnotatedMoveStructView).value

      const address = (((value as any).vec as AnnotatedMoveStructView).value[0] as Array<string>)[0]

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
    const sss = (
      (
        (states?.[0]?.decoded_value as AnnotatedMoveStructView).value[
          'value'
        ] as AnnotatedMoveStructView
      ).value['keys'] as AnnotatedMoveStructView
    ).value['handle'] as AnnotatedMoveStructView
    console.log(sss)
    // Maybe we should define the type?
    const tableId = (
      (
        (
          (states?.[0]?.decoded_value as AnnotatedMoveStructView).value[
            'value'
          ] as AnnotatedMoveStructView
        ).value['keys'] as AnnotatedMoveStructView
      ).value['handle'] as AnnotatedMoveStructView
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
}
