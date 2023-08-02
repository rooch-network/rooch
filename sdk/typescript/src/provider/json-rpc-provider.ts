// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochClient, Connection, localnetConnection } from "./rooch_client"
import { applyMixin } from "../utils"

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
  rpcClient?: RoochClient
}

const DEFAULT_OPTIONS: RpcProviderOptions = {
  versionCacheTimeoutInSeconds: 600,
}

export class JsonRpcProvider {
  public connection: Connection
  readonly client: RoochClient
  private rpcApiVersion: string | undefined
  private cacheExpiry: number | undefined

  constructor(
    connection: Connection = localnetConnection,
    public options: RpcProviderOptions = DEFAULT_OPTIONS,
  ) {
    this.connection = connection

    const opts = { ...DEFAULT_OPTIONS, ...options }
    this.options = opts

    this.client = opts.rpcClient ?? new RoochClient(this.connection)
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
      console.warn("Error fetching version number of the RPC API", err)
    }
    return void 0
  }
}

export interface JsonRpcProvider extends RoochClient {}

applyMixin(JsonRpcProvider, RoochClient, "client")
