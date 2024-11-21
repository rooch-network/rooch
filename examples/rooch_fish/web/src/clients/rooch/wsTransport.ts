import { 
  RoochTransport, 
  RoochHTTPTransportOptions, 
  RoochTransportRequestOptions,
  JsonRpcError
} from "@roochnetwork/rooch-sdk";

interface WsRequest {
  resolve: (value: any) => void
  reject: (error: Error) => void
  method: string
  params: unknown[]
  timestamp: number
}

export class RoochWebSocketTransport implements RoochTransport {
  #ws: WebSocket | null = null
  #requestId = 0
  #options: RoochHTTPTransportOptions
  #pendingRequests = new Map<number, WsRequest>()
  #reconnectAttempts = 0
  #maxReconnectAttempts = 5
  #reconnectDelay = 1000
  #connected = false
  #connecting = false

  constructor(options: RoochHTTPTransportOptions) {
    this.#options = options
    this.connect()
  }

  private connect() {
    if (this.#connecting || this.#connected) return

    this.#connecting = true
    const wsUrl = this.#options.url.replace(/^httpS/, 'wss')
    this.#ws = new WebSocket(wsUrl)

    this.#ws.onopen = () => {
      this.#connected = true
      this.#connecting = false
      this.#reconnectAttempts = 0
    }

    this.#ws.onclose = () => {
      this.#connected = false
      this.#ws = null
      this.handleReconnect()
    }

    this.#ws.onerror = (error) => {
      console.error('WebSocket error:', error)
    }

    this.#ws.onmessage = (event) => {
      try {
        const response = JSON.parse(event.data)
        this.handleResponse(response)
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }
  }

  private handleReconnect() {
    if (this.#reconnectAttempts >= this.#maxReconnectAttempts) {
      this.rejectAllPending(new Error('WebSocket connection failed'))
      return
    }

    this.#reconnectAttempts++
    setTimeout(() => this.connect(), this.#reconnectDelay * this.#reconnectAttempts)
  }

  private handleResponse(response: any) {
    const request = this.#pendingRequests.get(response.id)
    if (!request) return

    this.#pendingRequests.delete(response.id)

    if ('error' in response && response.error != null) {
      request.reject(new JsonRpcError(response.error.message, response.error.code))
    } else {
      request.resolve(response.result)
    }
  }

  private rejectAllPending(error: Error) {
    for (const request of this.#pendingRequests.values()) {
      request.reject(error)
    }
    this.#pendingRequests.clear()
  }

  async request<T>(input: RoochTransportRequestOptions): Promise<T> {
    if (!this.#connected) {
      throw new Error('WebSocket is not connected')
    }

    return new Promise((resolve, reject) => {
      const id = ++this.#requestId
      const request: WsRequest = {
        resolve,
        reject,
        method: input.method,
        params: input.params,
        timestamp: Date.now(),
      }

      this.#pendingRequests.set(id, request)
      this.#ws?.send(
        JSON.stringify({
          jsonrpc: '2.0',
          id,
          method: input.method,
          params: input.params,
        })
      )
    })
  }

  disconnect() {
    this.#ws?.close()
    this.#connected = false
    this.#connecting = false
    this.rejectAllPending(new Error('WebSocket disconnected'))
  }
}
