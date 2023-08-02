// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { HTTPTransport, RequestManager } from "@open-rpc/client-js"
import { JsonRpcClient } from "../generated/client"
import { applyMixin } from "../utils"

interface ConnectionOptions {
    url: string
	websocket?: string
}

export class Connection {
	#options: ConnectionOptions;
	constructor(options: ConnectionOptions) {
		this.#options = options;
	}

	get url() {
		return this.#options.url;
	}

	get websocket() {
		return this.#options.websocket || this.#options.url;
	}
}

export const localnetConnection = new Connection({
	url: 'http://localhost:50051',
});

// TODO: wapper JsonRpcClient or improved code generation, remove mixin
export class RoochClient {
  readonly client: JsonRpcClient

  readonly connection: Connection

  constructor(connection: Connection) {
    this.connection = connection

    this.client = new JsonRpcClient(new RequestManager([
        new HTTPTransport(connection.url, {
          headers: {
            "Content-Type": "application/json",
          },
        }),
      ]))
  }
}

export interface RoochClient extends JsonRpcClient {}

applyMixin(RoochClient, JsonRpcClient, "client")
