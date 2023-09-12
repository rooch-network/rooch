// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

interface ConnectionOptions {
  url: string
  websocket?: string
}

export class Connection {
  #options: ConnectionOptions

  constructor(options: ConnectionOptions) {
    this.#options = options
  }

  get url() {
    return this.#options.url
  }

  get websocket() {
    return this.#options.websocket || this.#options.url
  }
}

export const LocalNetConnection = new Connection({
  url: 'http://127.0.0.1:50051',
})
